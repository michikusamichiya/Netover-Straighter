pub mod encoders;

use crate::launch::ScreenOutputer::types::VideoEncoder;
use encoders::h264::H264Encoder;

/// エンコーダを生成するファクトリ関数。
/// 将来HWエンコードに切り替える場合はここだけ変更すればよい。
// ✅ 最新版
pub fn create_encoder() -> Result<Box<dyn VideoEncoder>, String> {
    let encoder = H264Encoder::new()  // 引数なし
        .map_err(|e| format!("Failed to create H264 encoder: {:?}", e))?;
    Ok(Box::new(encoder))
}