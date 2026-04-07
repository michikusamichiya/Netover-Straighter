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
    let (capture_client, native_channels) = initialize_audio_client(&audio_client, is_loopback)?;
    let native_channels = native_channels as usize;

    // ③ キャプチャ開始
    unsafe { audio_client.Start() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;

    // Opus は 2ch ステレオ、48000Hz、960フレーム(=20ms) を期待する
    // 1フレーム = 960 サンプル × 2ch = 1920 要素
    const OPUS_FRAME_SAMPLES: usize = 960;
    const OPUS_CHANNELS: usize = 2;
    const OPUS_FRAME_SIZE: usize = OPUS_FRAME_SAMPLES * OPUS_CHANNELS;

    // ステレオダウンミックス済みサンプルのバッファ
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

        // サンプルを読み取り、ステレオにダウンミックスしてバッファに積む
        if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
            // サイレントフレーム: ステレオ無音を追加
            sample_buf.extend(vec![0.0f32; num_frames_usize * OPUS_CHANNELS]);
        } else {
            // 実際のチャンネル数(native_channels)でスライスを作成
            let total_samples = num_frames_usize * native_channels;
            let raw = unsafe {
                std::slice::from_raw_parts(data_ptr as *const f32, total_samples)
            };

            // チャンネルダウンミックス: 全チャンネルの平均 → L/R に分配
            // native_channels == 2 の場合はそのまま通す
            for frame_i in 0..num_frames_usize {
                let offset = frame_i * native_channels;
                let frame_samples = &raw[offset..offset + native_channels];

                if native_channels == 1 {
                    // モノラル → ステレオ
                    let s = frame_samples[0];
                    sample_buf.push(s);
                    sample_buf.push(s);
                } else if native_channels == 2 {
                    // ステレオそのまま
                    sample_buf.push(frame_samples[0]);
                    sample_buf.push(frame_samples[1]);
                } else {
                    // マルチチャンネル → ステレオダウンミックス
                    // L: 偶数インデックスの平均、R: 奇数インデックスの平均
                    let half = native_channels / 2;
                    let l: f32 = frame_samples[..half].iter().sum::<f32>() / half as f32;
                    let r: f32 = frame_samples[half..].iter().sum::<f32>() / (native_channels - half) as f32;
                    sample_buf.push(l);
                    sample_buf.push(r);
                }
            }
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

/// IAudioClientを初期化してIAudioCaptureClientと実際のチャンネル数を返す
fn initialize_audio_client(
    audio_client: &IAudioClient,
    is_loopback: bool,
) -> Result<(IAudioCaptureClient, u16), PlatformError> {
    // デバイスのネイティブミックスフォーマットを取得してそのまま使う
    // （SHARED モードでは GetMixFormat() が返すフォーマット以外は通常使えない）
    let mix_format = unsafe { audio_client.GetMixFormat() }
        .map_err(|e| PlatformError::APIError(e.code().0))?;

    let (native_channels, sample_rate, bits) = unsafe {
        let channels = (*mix_format).nChannels;
        let sr = (*mix_format).nSamplesPerSec;
        let bits = (*mix_format).wBitsPerSample;
        (channels, sr, bits)
    };

    log::info!(
        "Audio format: {}Hz, {}ch, {}bit",
        sample_rate,
        native_channels,
        bits,
    );

    let stream_flags: u32 = if is_loopback {
        AUDCLNT_STREAMFLAGS_LOOPBACK
    } else {
        0
    };

    unsafe {
        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            stream_flags,
            1_000_000,
            0,
            mix_format,
            None,
        )
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    // IAudioCaptureClient取得
    let capture_client: IAudioCaptureClient = unsafe {
        audio_client.GetService()
    }.map_err(|e| PlatformError::APIError(e.code().0))?;

    Ok((capture_client, native_channels))
}