pub mod platform;

use std::sync::Arc;
use types::AudioCaptureLoop;

pub fn create_audio_capture_loop() -> Arc<dyn AudioCaptureLoop + Send + Sync> {
    #[cfg(target_os = "windows")]
    {
        use crate::launch::ScreenOutputer::AudioCapture::platform::windows::WindowsAudioCaptureLoop;
        return Arc::new(WindowsAudioCaptureLoop {});
    }
    #[cfg(target_os = "linux")]
    {
        // use crate::AudioCapture::platform::linux::LinuxAudioCaptureLoop;
        // return Arc::new(LinuxAudioCaptureLoop);
        unimplemented!("Linux is not supported yet")
    }
    #[cfg(target_os = "macos")]
    {
        // use crate::AudioCapture::platform::mac::MacAudioCaptureLoop;
        // return Arc::new(MacAudioCaptureLoop);
        unimplemented!("MacOS is not supported yet")
    }
}