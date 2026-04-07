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
    // Windows がリサンプリング＆ダウンミックスを行うため、
    // キャプチャは常に 48000Hz / 2ch / f32 で返ってくる
    let capture_client = initialize_audio_client(&audio_client, is_loopback)?;

    // ③ キャプチャ開始
    unsafe { audio_client.Start() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;

    // Opus: 2ch ステレオ、48000Hz、960サンプル(=20ms) = 1920 要素/フレーム
    const OPUS_FRAME_SAMPLES: usize = 960;
    const OPUS_CHANNELS: usize = 2;
    const OPUS_FRAME_SIZE: usize = OPUS_FRAME_SAMPLES * OPUS_CHANNELS;

    let mut sample_buf: Vec<f32> = Vec::with_capacity(OPUS_FRAME_SIZE * 4);

    while running.load(Ordering::SeqCst) {
        let packet_size = unsafe { capture_client.GetNextPacketSize() }
            .map_err(|e| PlatformError::APIError(e.code().0))?;

        if packet_size == 0 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            continue;
        }

        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut num_frames = 0u32;
        let mut flags = 0u32;
        unsafe {
            capture_client.GetBuffer(&mut data_ptr, &mut num_frames, &mut flags, None, None)
        }.map_err(|e| PlatformError::APIError(e.code().0))?;

        let num_frames_usize = num_frames as usize;

        // AUTOCONVERTPCM により Windows 側で 48000Hz/2ch/f32 に変換済み
        // → 2ch 固定で直接読み取るだけでよい
        if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
            sample_buf.extend(vec![0.0f32; num_frames_usize * OPUS_CHANNELS]);
        } else {
            let samples = unsafe {
                std::slice::from_raw_parts(data_ptr as *const f32, num_frames_usize * OPUS_CHANNELS)
            };
            sample_buf.extend_from_slice(samples);
        }
        unsafe { capture_client.ReleaseBuffer(num_frames) }
            .map_err(|e| PlatformError::APIError(e.code().0))?;

        // 960サンプル×2ch (1920要素) 溜まったらエンコード
        while sample_buf.len() >= OPUS_FRAME_SIZE {
            let chunk: Vec<f32> = sample_buf.drain(..OPUS_FRAME_SIZE).collect();
            let frame = AudioFrame { samples: chunk, sample_rate: 48000, channels: 2 };
            if let Ok(encoded) = encoder.encode(&frame) {
                on_frame(encoded);
            }
        }
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
/// AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM + SRC_QUALITY により
/// Windows が自動でリサンプリング＆ダウンミックスを行う。
/// 呼び出し後は常に 48000Hz / 2ch / IEEE_FLOAT 32bit でデータが返る。
fn initialize_audio_client(
    audio_client: &IAudioClient,
    is_loopback: bool,
) -> Result<IAudioCaptureClient, PlatformError> {
    // デバイスのネイティブフォーマットをログだけのために読む
    let mix_format = unsafe { audio_client.GetMixFormat() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;
    unsafe {
        let sr   = std::ptr::read_unaligned(std::ptr::addr_of!((*mix_format).nSamplesPerSec));
        let ch   = std::ptr::read_unaligned(std::ptr::addr_of!((*mix_format).nChannels));
        let bits = std::ptr::read_unaligned(std::ptr::addr_of!((*mix_format).wBitsPerSample));
        log::info!("Native audio format: {}Hz, {}ch, {}bit", sr, ch, bits);
    }

    // Opus に合わせた目標フォーマット: 48000Hz / 2ch / IEEE_FLOAT 32bit
    let desired_format = WAVEFORMATEX {
        wFormatTag: 3u16, // WAVE_FORMAT_IEEE_FLOAT
        nChannels: 2,
        nSamplesPerSec: 48000,
        nAvgBytesPerSec: 48000 * 2 * 4,
        nBlockAlign: 2 * 4,
        wBitsPerSample: 32,
        cbSize: 0,
    };

    // AUTOCONVERTPCM: Windows Audio Engine がフォーマット変換を担当
    // SRC_QUALITY:    高品質なリサンプラーを使用（44100→48000 等）
    const AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM: u32 = 0x80000000;
    const AUDCLNT_STREAMFLAGS_SRC_QUALITY: u32    = 0x08000000;

    let mut stream_flags: u32 = AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_QUALITY;
    if is_loopback {
        stream_flags |= AUDCLNT_STREAMFLAGS_LOOPBACK;
    }

    unsafe {
        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            stream_flags,
            1_000_000,
            0,
            &desired_format as *const WAVEFORMATEX,
            None,
        )
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    log::info!("Audio initialized: 48000Hz, 2ch, 32bit (AUTOCONVERTPCM)");

    let capture_client: IAudioCaptureClient = unsafe {
        audio_client.GetService()
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    Ok(capture_client)
}