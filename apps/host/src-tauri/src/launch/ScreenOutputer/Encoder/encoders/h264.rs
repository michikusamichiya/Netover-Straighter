use openh264::encoder::{BitRate, Encoder, EncoderConfig, FrameRate, IntraFramePeriod};
use openh264::formats::YUVSource;
use openh264::OpenH264API;
use crate::launch::ScreenOutputer::types::{I420Frame, VideoEncoder, PlatformError};

// ===========================
// I420FrameのラッパーとYUVSource実装
// ===========================

// types.rsをopenh264に依存させないために、h264.rs内でラッパー型を定義する。
// I420Frameの参照を持つだけで、データのコピーは発生しない。
struct I420FrameRef<'a>(&'a I420Frame);

impl<'a> YUVSource for I420FrameRef<'a> {
    // 画像のサイズを返す
    fn dimensions(&self) -> (usize, usize) {
        (self.0.width as usize, self.0.height as usize)
    }

    // 各プレーンの1行あたりのバイト数（ストライド）を返す。
    // パディングなしの密なレイアウトなのでそのままwidthが使える。
    // UVはダウンサンプリングされているので width/2 になる。
    fn strides(&self) -> (usize, usize, usize) {
        let y_stride = self.0.width as usize;
        let uv_stride = (self.0.width / 2) as usize;
        (y_stride, uv_stride, uv_stride)
    }

    fn y(&self) -> &[u8] { &self.0.y }
    fn u(&self) -> &[u8] { &self.0.u }
    fn v(&self) -> &[u8] { &self.0.v }
}

// ===========================
// H264Encoder
// ===========================

pub struct H264Encoder {
    /// openh264のエンコーダ本体
    encoder: Encoder,
}

impl H264Encoder {
    /// エンコーダを作成する
    pub fn new() -> Result<Self, PlatformError> {
        let config = EncoderConfig::new()
            // BitRate型でビットレートを指定する
            .bitrate(BitRate::from_bps(2_000_000))
            // メソッド名はmax_frame_rate（アンダースコアあり）
            .max_frame_rate(FrameRate::from_hz(60.0))
            .intra_frame_period(IntraFramePeriod::from_num_frames(300));

        // with_config()は存在せず、with_api_config()を使う。
        // OpenH264API::from_source()でAPIを取得してから渡す。
        let api = OpenH264API::from_source();
        let encoder = Encoder::with_api_config(api, config)
            .map_err(|_| PlatformError::PrepareError)?;

        Ok(Self { encoder })
    }
}

impl VideoEncoder for H264Encoder {
    fn encode(&mut self, frame: &I420Frame) -> Result<Vec<u8>, PlatformError> {
        // I420FrameをラッパーでYUVSourceとして扱えるようにする
        let yuv = I420FrameRef(frame);

        // encode()はBitstreamを返す（NALユニットの集合体）
        let bitstream = self.encoder
            .encode(&yuv)
            .map_err(|_| PlatformError::APIError(0))?;

        // NALユニット全体を1つのVec<u8>として書き出す
        let mut buf = Vec::new();
        bitstream.write_vec(&mut buf);

        Ok(buf)
    }
}