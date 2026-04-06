use crate::launch::InputInjector::platform::windows::WindowsKey;
use crate::launch::InputInjector::platform::linux::LinuxKey;
use crate::launch::InputInjector::platform::mac::MacKey;
use crate::launch::ScreenOutputer::types::Screen;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug)]
pub enum PlatformError {
  APIError(i32),
  TypeError,
  PrepareError,
  ExistError,
}

pub enum LogicalKey {
  WindowsKey(WindowsKey),
  LinuxKey(LinuxKey),
  MacKey(MacKey),
}

pub struct InputStat {
  pub screen: Screen,
  // pub injector: Arc<dyn InputInjector>,
  pub keymap: Arc<HashMap<String, LogicalKey>>
}

pub trait MouseInjector {
  fn clickup(&self, button: u32) -> Result<(), PlatformError>;
  fn clickdown(&self, button: u32) -> Result<(), PlatformError>;
  fn moveabsolute(&self, x: f32, y: f32, bp: (i32, i32, i32, i32)) -> Result<(), PlatformError>;
  fn moverelative(&self, dx: i32, dy: i32) -> Result<(), PlatformError>;
  fn wheel(&self, delta_x: f32, delta_y: f32) -> Result<(), PlatformError>;
}
pub trait KeyboardBackend {
  fn key_down(&self, key: &Option<&LogicalKey>) -> Result<(), PlatformError>;
  fn key_up(&self, key: &Option<&LogicalKey>) -> Result<(), PlatformError>;
}
#[async_trait]
pub trait InputInjectorGeneral {
  async fn handle_input(&self, stat: &InputStat, string: String) -> Result<(), PlatformError>;
  // fn get_keymap_path(&self) -> String;
  fn load_keymap(&self) -> Result<HashMap<String, LogicalKey>, PlatformError>;
  fn new() -> Self where Self: Sized;
}
#[async_trait]
pub trait InputInjector: MouseInjector + KeyboardBackend + InputInjectorGeneral + Send + Sync {}