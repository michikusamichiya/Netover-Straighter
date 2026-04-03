use std::sync::Arc;
use types::InputInjector;

pub mod types;
pub mod platform;

pub fn create_injector() -> Arc<dyn InputInjector> {
    #[cfg(target_os = "windows")]
    {
        use crate::InputInjector::platform::windows::WindowsInputInjector;
        return Arc::new(WindowsInputInjector);
    }
    #[cfg(target_os = "linux")]
    {
        // return Arc::new(LinuxInputInjector::new());
        unimplemented!("Linux is not supported yet");
    }
    #[cfg(target_os = "macos")]
    {
        // return Arc::new(MacInputInjector::new());
        unimplemented!("MacOS is not supported yet");
    }
    // TODO: 実装
}