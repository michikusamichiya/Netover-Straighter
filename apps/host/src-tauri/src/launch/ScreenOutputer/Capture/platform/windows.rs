use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::IDXGIOutput;
use windows::Win32::Graphics::Dxgi::DXGI_OUTPUT_DESC;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::*;
use windows::core::ComInterface; // cast() メソッドを使うために必要
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use rayon::prelude::*;
use crate::launch::ScreenOutputer::Capture::types::*;

// ===========================
// WindowsCaptureWay
// ===========================

pub struct WindowsCaptureWay {}

impl CaptureWayGeneral for WindowsCaptureWay {
    fn get_screens(&self) -> Result<Vec<(Screen, NativeScreen)>, PlatformError> {
        use windows::Win32::Graphics::Dxgi::*;

        let factory: IDXGIFactory1 = unsafe {
            CreateDXGIFactory1()
                .map_err(|e| PlatformError::APIError(e.code().0))?
        };

        let mut screens = Vec::new();
        let mut adapter_idx = 0u32;

        loop {
            let adapter = match unsafe { factory.EnumAdapters1(adapter_idx) } {
                Ok(a) => a,
                Err(_) => break,
            };

            let mut output_idx = 0u32;
            loop {
                let output: IDXGIOutput = match unsafe { adapter.EnumOutputs(output_idx) } {
                    Ok(o) => o,
                    Err(_) => break,
                };

                let mut desc = DXGI_OUTPUT_DESC::default();
                unsafe { output.GetDesc(&mut desc) }
                    .map_err(|e| PlatformError::APIError(e.code().0))?;

                let rect = desc.DesktopCoordinates;
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                let name = String::from_utf16_lossy(
                    &desc.DeviceName
                        .iter()
                        .take_while(|&&c| c != 0)
                        .cloned()
                        .collect::<Vec<_>>(),
                );

                let primary = rect.left == 0 && rect.top == 0;
                let id = format!("win:{}:{}", adapter_idx, output_idx);

                screens.push((
                    Screen { id, name, width, height, x: rect.left, y: rect.top, primary },
                    NativeScreen::Windows { adapter_idx, output_idx },
                ));

                output_idx += 1;
            }

            adapter_idx += 1;
        }

        Ok(screens)
    }
}

// ===========================
// WindowsCaptureLoop
// ===========================

pub struct WindowsCaptureLoop {
    /// stop()からループを止めるためのフラグ
    running: Arc<AtomicBool>,
}

impl WindowsCaptureLoop {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl CaptureLoop for WindowsCaptureLoop {
    fn start(
        &mut self,
        target: &NativeScreen,
        mut encoder: Box<dyn VideoEncoder>,
        on_frame: Box<dyn Fn(Vec<u8>) + Send>,
    ) -> Result<(), PlatformError> {
        let (adapter_idx, output_idx) = match target {
            NativeScreen::Windows { adapter_idx, output_idx } => (*adapter_idx, *output_idx),
            _ => return Err(PlatformError::TypeError),
        };

        // 二重起動防止
        if self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::ExistError);
        }
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        // Desktop Duplication APIはCOMスレッドアフィニティの制約があるため
        // tokio::spawnではなくstd::thread::spawnで専用スレッドを立てる
        std::thread::spawn(move || {
            let result = capture_loop_inner(
                adapter_idx,
                output_idx,
                &running,
                &mut *encoder,
                &on_frame,
            );
            if let Err(e) = result {
                log::error!("CaptureLoop exited with error: {:?}", e);
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

/// ループのメイン処理を関数に切り出してエラーを返せるようにする
fn capture_loop_inner(
    adapter_idx: u32,
    output_idx: u32,
    running: &AtomicBool,
    encoder: &mut dyn VideoEncoder,
    on_frame: &dyn Fn(Vec<u8>),
) -> Result<(), PlatformError> {
    let (device, context) = create_d3d11_device(adapter_idx)?;
    let duplication = create_duplication(&device, adapter_idx, output_idx)?;
    let (width, height) = get_output_size(adapter_idx, output_idx)?;
    let staging = create_staging_texture(&device, width, height)?;
    while running.load(Ordering::SeqCst) {
        // let t0 = std::time::Instant::now();

        // ① フレーム取得（タイムアウト33ms = 約30fps上限）
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        match unsafe { duplication.AcquireNextFrame(33, &mut frame_info, &mut resource) } {
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => continue,
            Err(e) => return Err(PlatformError::APIError(e.code().0)),
            Ok(_) => {}
        }
        // let t1 = std::time::Instant::now();

        // ② IDXGIResource → ID3D11Texture2D にキャスト
        let frame_texture: ID3D11Texture2D = match resource
            .as_ref()
            .and_then(|r| r.cast::<ID3D11Texture2D>().ok())
        {
            Some(t) => t,
            None => {
                unsafe { duplication.ReleaseFrame().ok() };
                continue;
            }
        };

        // ③ GPU→CPU コピー（StagingTextureへ）
        unsafe { context.CopyResource(&staging, &frame_texture) };

        // ④ CPUでマップしてBGRAバイト列を読み取る
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        if let Err(e) = unsafe {
            context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
        } {
            unsafe { duplication.ReleaseFrame().ok() };
            return Err(PlatformError::APIError(e.code().0));
        }

        let bgra_data: Vec<u8> = unsafe {
            let ptr = mapped.pData as *const u8;
            let stride = mapped.RowPitch as usize;
            let w = width as usize;
            let h = height as usize;
            let mut buf = Vec::with_capacity(w * h * 4);
            for row in 0..h {
                let row_ptr = ptr.add(row * stride);
                buf.extend_from_slice(std::slice::from_raw_parts(row_ptr, w * 4));
            }
            buf
        };

        // 必ずUnmapとReleaseFrameを呼ぶ（呼ばないと次フレームが取れない）
        unsafe { context.Unmap(&staging, 0) };
        unsafe { duplication.ReleaseFrame().ok() };
        // let t2 = std::time::Instant::now();

        // ⑤ BGRA → I420変換
        let i420 = bgra_to_i420(&bgra_data, width, height)?;
        // let t3 = std::time::Instant::now();

        // ⑥ エンコード
        let encoded = encoder.encode(&i420)?;
        // let t4 = std::time::Instant::now();

        // log::info!(
        //     "acquire={}ms, gpu_to_cpu={}ms, convert={}ms, encode={}ms, total={}ms",
        //     t1.duration_since(t0).as_millis(),
        //     t2.duration_since(t1).as_millis(),
        //     t3.duration_since(t2).as_millis(),
        //     t4.duration_since(t3).as_millis(),
        //     t4.duration_since(t0).as_millis(),
        // );

        on_frame(encoded);
    }
    Ok(())
}

// ===========================
// ヘルパー関数群
// ===========================

/// 指定アダプタでD3D11デバイスとコンテキストを作成する
fn create_d3d11_device(adapter_idx: u32) -> Result<(ID3D11Device, ID3D11DeviceContext), PlatformError> {
    let factory: IDXGIFactory1 = unsafe {
        CreateDXGIFactory1().map_err(|e| PlatformError::APIError(e.code().0))?
    };
    let adapter = unsafe {
        factory.EnumAdapters1(adapter_idx).map_err(|e| PlatformError::APIError(e.code().0))?
    };

    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let feature_levels = [windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0];

    unsafe {
        D3D11CreateDevice(
            &adapter,                          // キャプチャ対象と同じアダプタを使う
            D3D_DRIVER_TYPE_UNKNOWN,           // adapter指定時はUNKNOWN必須
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,  // DXGIとのBGRA互換を有効化
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        ).map_err(|e| PlatformError::APIError(e.code().0))?;
    }

    Ok((device.unwrap(), context.unwrap()))
}

/// Desktop Duplication (IDXGIOutputDuplication) を作成する
fn create_duplication(
    device: &ID3D11Device,
    adapter_idx: u32,
    output_idx: u32,
) -> Result<IDXGIOutputDuplication, PlatformError> {
    let factory: IDXGIFactory1 = unsafe {
        CreateDXGIFactory1().map_err(|e| PlatformError::APIError(e.code().0))?
    };
    let adapter = unsafe {
        factory.EnumAdapters1(adapter_idx).map_err(|e| PlatformError::APIError(e.code().0))?
    };
    let output: IDXGIOutput = unsafe {
        adapter.EnumOutputs(output_idx).map_err(|e| PlatformError::APIError(e.code().0))?
    };
    // IDXGIOutput → IDXGIOutput1 にキャスト
    // DuplicateOutput() は IDXGIOutput1 にしか存在しないため昇格が必要
    let output1: IDXGIOutput1 = output.cast::<IDXGIOutput1>()
        .map_err(|e: windows::core::Error| PlatformError::APIError(e.code().0))?;

    // ID3D11Device → IDXGIDevice にキャスト
    // DuplicateOutput() の引数がIDXGIDeviceを要求するため
    let dxgi_device: IDXGIDevice = device.cast::<IDXGIDevice>()
        .map_err(|e: windows::core::Error| PlatformError::APIError(e.code().0))?;

    unsafe {
        output1.DuplicateOutput(&dxgi_device)
            .map_err(|e: windows::core::Error| PlatformError::APIError(e.code().0))
    }
}

/// 指定Outputの解像度を取得する
fn get_output_size(adapter_idx: u32, output_idx: u32) -> Result<(u32, u32), PlatformError> {
    let factory: IDXGIFactory1 = unsafe {
        CreateDXGIFactory1().map_err(|e| PlatformError::APIError(e.code().0))?
    };
    let adapter = unsafe {
        factory.EnumAdapters1(adapter_idx).map_err(|e| PlatformError::APIError(e.code().0))?
    };
    let output: IDXGIOutput = unsafe {
        adapter.EnumOutputs(output_idx).map_err(|e| PlatformError::APIError(e.code().0))?
    };

    let mut desc = DXGI_OUTPUT_DESC::default();
    unsafe { output.GetDesc(&mut desc).map_err(|e| PlatformError::APIError(e.code().0))? };

    let rect = desc.DesktopCoordinates;
    Ok(((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32))
}

/// GPU→CPUコピー用のStagingテクスチャを作成する
fn create_staging_texture(
    device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Result<ID3D11Texture2D, PlatformError> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,  // DXGIのデフォルトフォーマット（BGRA 8bit）
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_STAGING,           // CPUがReadできるようにSTAGING指定
        // フラグ類はwindows 0.52ではu32として渡す必要がある
        BindFlags: 0,                         // StagingテクスチャはGPUへのバインド不要
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32, // CPUからの読み取りアクセスを許可
        MiscFlags: 0,                         // 特殊フラグなし
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        device.CreateTexture2D(&desc, None, Some(&mut texture))
            .map_err(|e| PlatformError::APIError(e.code().0))?;
    }

    Ok(texture.unwrap())
}

/// BGRAバイト列をI420Frameに変換する（BT.601係数・rayon並列化）
fn bgra_to_i420(bgra: &[u8], width: u32, height: u32) -> Result<I420Frame, PlatformError> {
    let w = width as usize;
    let h = height as usize;
    let uv_w = w / 2;
    let uv_h = h / 2;

    let mut y_plane = vec![0u8; w * h];
    // UVプレーンはYプレーンと独立して並列処理するため別々に確保
    let mut u_plane = vec![0u8; uv_w * uv_h];
    let mut v_plane = vec![0u8; uv_w * uv_h];

    // Yプレーン: 全ピクセルを行単位で並列処理
    // par_chunks_mut(w) で1行ずつスレッドに分配する
    y_plane
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(row, y_row)| {
            for col in 0..w {
                let idx = (row * w + col) * 4;
                let b = bgra[idx] as i32;
                let g = bgra[idx + 1] as i32;
                let r = bgra[idx + 2] as i32;
                let y = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
                y_row[col] = y.clamp(0, 255) as u8;
            }
        });

    // UVプレーン: 偶数行のみ・2ピクセルおきに処理
    // UVはYの1/4のサイズなのでuv_h行 × uv_w列
    // par_chunks_mut(uv_w) で1行ずつ並列処理
    u_plane
        .par_chunks_mut(uv_w)
        .zip(v_plane.par_chunks_mut(uv_w))
        .enumerate()
        .for_each(|(uv_row, (u_row, v_row))| {
            let src_row = uv_row * 2; // 元画像の対応する行（偶数行）
            for uv_col in 0..uv_w {
                let src_col = uv_col * 2; // 元画像の対応する列（偶数列）
                let idx = (src_row * w + src_col) * 4;
                let b = bgra[idx] as i32;
                let g = bgra[idx + 1] as i32;
                let r = bgra[idx + 2] as i32;
                u_row[uv_col] = (((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128).clamp(0, 255) as u8;
                v_row[uv_col] = (((112 * r - 94 * g - 18 * b + 128) >> 8) + 128).clamp(0, 255) as u8;
            }
        });

    Ok(I420Frame { width, height, y: y_plane, u: u_plane, v: v_plane })
}