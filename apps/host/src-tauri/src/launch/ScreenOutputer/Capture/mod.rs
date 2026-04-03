pub mod platform;

use std::sync::Arc;
use types::CaptureWayGeneral;

use crate::launch::ScreenOutputer::types;

pub fn create_capture_way() -> Arc<dyn CaptureWayGeneral + Send + Sync> {
    #[cfg(target_os = "windows")]
    {
        use crate::launch::ScreenOutputer::Capture::platform::windows::WindowsCaptureWay;
        return Arc::new(WindowsCaptureWay {});
    }
    #[cfg(target_os = "linux")]
    {
        // use crate::CaptureWay::platform::linux::LinuxCaptureWay;
        // return Arc::new(LinuxCaptureWay);
        unimplemented!("Linux is not supported yet")
    }
    #[cfg(target_os = "macos")]
    {
        // use crate::CaptureWay::platform::mac::MacCaptureWay;
        // return Arc::new(MacCaptureWay);
        unimplemented!("MacOS is not supported yet")
    }
}