use std::collections::HashMap;

#[derive(Debug)]
pub enum PlatformError {
  APIError(i32),
  TypeError,
  PrepareError,
  ExistError,
}
#[derive(Clone)]
pub struct Screen {
  pub id: String,
  pub name: String,
  pub width: u32,
  pub height: u32,
  pub x: i32,
  pub y: i32,
  pub primary: bool,
}
impl Screen {
  pub fn get_base_point(&self) -> (i32, i32, i32, i32) {
    (self.x, self.y, self.x + self.width as i32, self.y + self.height as i32)
  }
}
#[derive(Clone)]
pub enum NativeScreen {
  Windows { adapter_idx: u32, output_idx: u32 },
  Mac { display_id: u32 },
  Linux { internal_id: String },
}
pub struct ScreenManager {
  pub screens: Vec<Screen>,
  pub now_screen: Screen,
  pub native_map: HashMap<String, NativeScreen>, // Screen.id -> NativeScreen
}
// I420FrameはWebRTCで使われるピクセルフォーマット。
pub struct I420Frame {
  pub width: u32,
  pub height: u32,
  pub y: Vec<u8>,
  pub u: Vec<u8>,
  pub v: Vec<u8>,
}
pub trait CaptureWayGeneral: Send + Sync {
  // fn new() -> Self where Self: Sized;
  fn get_screens(&self) -> Result<Vec<(Screen, NativeScreen)>, PlatformError>;
  // fn get_primary_screen(&self) -> Result<Screen, PlatformError>;
}

pub trait VideoEncoder : Send {
  fn encode(&mut self, frame: &I420Frame) -> Result<Vec<u8>, PlatformError>;
}
pub trait CaptureLoop: Send {
  fn start(
    &mut self,
    target: &NativeScreen,
    encoder: Box<dyn VideoEncoder>,
    on_frame: Box<dyn Fn(Vec<u8>) + Send>,
  ) -> Result<(), PlatformError>;
  fn stop(&mut self);
}