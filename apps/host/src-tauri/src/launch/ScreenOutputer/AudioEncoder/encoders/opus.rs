// src/launch/ScreenOutputer/AudioEncoder/opus.rs
use audiopus::{coder::Encoder, Application, Channels, SampleRate};
use crate::launch::ScreenOutputer::types::{AudioEncoder, AudioFrame, PlatformError};

pub struct OpusAudioEncoder {
    encoder: Encoder,
}

impl OpusAudioEncoder {
    pub fn new() -> Result<Self, PlatformError> {
        let encoder = Encoder::new(
            SampleRate::Hz48000,
            Channels::Stereo,
            Application::Audio, // 音楽・システム音向け。VoIPならVoIPに変える
        ).map_err(|_| PlatformError::EncoderError)?;
        Ok(Self { encoder })
    }
}

impl AudioEncoder for OpusAudioEncoder {
    fn encode(&mut self, frame: &AudioFrame) -> Result<Vec<u8>, PlatformError> {
        // audiopusはi16入力のため f32→i16 変換
        let pcm_i16: Vec<i16> = frame.samples.iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        let mut output = vec![0u8; 4000]; // Opusの最大フレームサイズ
        let len = self.encoder
            .encode(&pcm_i16, &mut output)
            .map_err(|_| PlatformError::EncoderError)?;
        output.truncate(len);
        Ok(output)
    }
}