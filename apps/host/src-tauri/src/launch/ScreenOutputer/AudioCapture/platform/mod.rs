#[cfg(target_os = "windows")]
pub mod windows;

use std::sync::Arc;
use crate::launch::ScreenOutputer::types::AudioCaptureLoop;

pub fn create_audio_capture_loop() -> Box<dyn AudioCaptureLoop> {
    #[cfg(target_os = "windows")]
    {
        use crate::launch::ScreenOutputer::AudioCapture::platform::windows::WindowsAudioCaptureLoop;
        let capture_loop: Box<dyn AudioCaptureLoop> = Box::new(WindowsAudioCaptureLoop::new());
        return capture_loop;
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