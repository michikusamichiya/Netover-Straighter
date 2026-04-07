use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::core::PCWSTR;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::launch::ScreenOutputer::types::*;

// ===========================
// WindowsAudioCaptureLoop
// ===========================

pub struct WindowsAudioCaptureLoop {
    running: Arc<AtomicBool>,
}

impl WindowsAudioCaptureLoop {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AudioCaptureLoop for WindowsAudioCaptureLoop {
    fn start(
        &mut self,
        target: &NativeAudioDevice,
        mut encoder: Box<dyn AudioEncoder>,
        on_frame: Box<dyn Fn(Vec<u8>) + Send>,
    ) -> Result<(), PlatformError> {
        let device_id = match target {
            NativeAudioDevice::Windows { device_id } => device_id.clone(),
            _ => return Err(PlatformError::TypeError),
        };

        // 二重起動防止（VideoのWindowsCaptureLoopと同じパターン）
        if self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::ExistError);
        }
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        // WASAPIもCOMスレッドアフィニティの制約があるため std::thread::spawn 必須
        // tokio::spawn は不可（VideoのDesktop Duplicationと同じ理由）
        std::thread::spawn(move || {
            let result = audio_capture_loop_inner(
                &device_id,
                &running,
                &mut *encoder,
                &on_frame,
            );
            if let Err(e) = result {
                log::error!("AudioCaptureLoop exited with error: {:?}", e);
            }
            running.store(false, Ordering::SeqCst);
        });

        Ok(())
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// ===========================
// キャプチャループ本体
// ===========================

fn audio_capture_loop_inner(
    device_id: &str,
    running: &AtomicBool,
    encoder: &mut dyn AudioEncoder,
    on_frame: &dyn Fn(Vec<u8>),
) -> Result<(), PlatformError> {
    // ① COM初期化（このスレッド専用。VideoのD3D11と同じ理由でスレッドローカル）
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .map_err(|e| PlatformError::APIError(e.code().0))?;

    let (audio_client, is_loopback) = create_audio_client(device_id)?;
    let capture_client = initialize_audio_client(&audio_client, is_loopback)?;

    // ③ キャプチャ開始
    unsafe { audio_client.Start() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;

    while running.load(Ordering::SeqCst) {
        // ④ バッファにデータが来るまで待つ
        // VideoのAcquireNextFrame(33ms)に相当するポーリング
        let packet_size = unsafe { capture_client.GetNextPacketSize() }
            .map_err(|e| PlatformError::APIError(e.code().0))?;

        if packet_size == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        // ⑤ バッファ取得
        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut num_frames = 0u32;
        let mut flags = 0u32;
        unsafe {
            capture_client.GetBuffer(
                &mut data_ptr,
                &mut num_frames,
                &mut flags,
                None,
                None,
            )
        }.map_err(|e| PlatformError::APIError(e.code().0))?;

        // ⑥ AUDCLNT_BUFFERFLAGS_SILENT は無音フレーム（スキップ可）
        let encoded = if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
            // 無音フレームはゼロPCMとして扱う
            let silent_frame = AudioFrame {
                samples: vec![0.0f32; num_frames as usize * 2], // stereo想定
                sample_rate: 48000,
                channels: 2,
            };
            encoder.encode(&silent_frame)?
        } else {
            // ⑦ *mut u8 → &[f32] に変換
            // WASAPIはIEEE_FLOAT形式で初期化しているのでf32直読みできる
            let samples = unsafe {
                std::slice::from_raw_parts(
                    data_ptr as *const f32,
                    num_frames as usize * 2, // stereo: channels=2
                ).to_vec()
            };

            // ⑧ VideoのReleaseFrame()に相当 → 必ずGetBufferの直後に呼ぶ
            unsafe { capture_client.ReleaseBuffer(num_frames) }
                .map_err(|e| PlatformError::APIError(e.code().0))?;

            let frame = AudioFrame { samples, sample_rate: 48000, channels: 2 };
            encoder.encode(&frame)?
        };

        // silentの場合はReleaseBufferをここで呼ぶ
        // （上のelse分岐では先に呼んでいるため条件分岐が必要）
        if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
            unsafe { capture_client.ReleaseBuffer(num_frames) }
                .map_err(|e| PlatformError::APIError(e.code().0))?;
        }

        on_frame(encoded);
    }

    unsafe { audio_client.Stop() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;
    unsafe { CoUninitialize() };

    Ok(())
}

// ===========================
// ヘルパー関数群
// ===========================

/// デバイスIDからIAudioClientを作成する
/// device_id が "loopback" の場合はシステム音キャプチャ用デバイスを選択
fn create_audio_client(device_id: &str) -> Result<(IAudioClient, bool), PlatformError> {
    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    let (mm_device, is_loopback) = if device_id == "loopback" {
        // システム音キャプチャ: デフォルトレンダリングデバイスのloopback
        // （マイクではなくスピーカーの出力音を取る）
        let dev = unsafe {
            enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
        }.map_err(|e| PlatformError::APIError(e.code().0))?;
        (dev, true)
    } else {
        // 通常のマイク等: IDで指定
        let id_wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        let dev = unsafe {
            enumerator.GetDevice(PCWSTR(id_wide.as_ptr()))
        }.map_err(|e| PlatformError::APIError(e.code().0))?;
        (dev, false)
    };

    let audio_client: IAudioClient = unsafe {
        mm_device.Activate(CLSCTX_ALL, None)
    }.map_err(|e: windows::core::Error| PlatformError::APIError(e.code().0))?;

    Ok((audio_client, is_loopback))
}

/// IAudioClientを初期化してIAudioCaptureClientを返す
fn initialize_audio_client(
    audio_client: &IAudioClient,
    is_loopback: bool,
) -> Result<IAudioCaptureClient, PlatformError> {
    // ② フォーマット指定: 48kHz, 2ch, IEEE_FLOAT 32bit
    // これでGetBuffer後にf32直読みできる（VideoのBGRAに相当）
    let format = WAVEFORMATEX {
        wFormatTag: 3 as u16, // WAVE_FORMAT_IEEE_FLOAT
        nChannels: 2,
        nSamplesPerSec: 48000,
        nAvgBytesPerSec: 48000 * 2 * 4, // sampleRate * channels * bytesPerSample
        nBlockAlign: 2 * 4,             // channels * bytesPerSample
        wBitsPerSample: 32,
        cbSize: 0,
    };

    let stream_flags: u32 = if is_loopback {
        AUDCLNT_STREAMFLAGS_LOOPBACK
    } else {
        0
    };
    unsafe {
        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            stream_flags, // u32をそのまま渡す
            1_000_000,
            0,
            &format,
            None,
        )
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    // IAudioCaptureClient取得
    // VideoでIDXGIOutputDuplicationを取得するのに相当するステップ
    let capture_client: IAudioCaptureClient = unsafe {
        audio_client.GetService()
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    Ok(capture_client)
}