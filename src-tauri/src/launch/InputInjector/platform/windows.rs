use async_trait::async_trait;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::launch::InputInjector::types::*;
use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
pub struct WindowsKey {
  pub scancode: u16,
  pub extended: bool,
}

pub struct WindowsInputInjector;

impl MouseInjector for WindowsInputInjector {
  fn clickup(&self, button: u32) -> Result<(), PlatformError> {
    let flags = match button {
      0 => MOUSEEVENTF_LEFTUP,
      1 => MOUSEEVENTF_RIGHTUP,
      2 => MOUSEEVENTF_MIDDLEUP,
      _ => return Err(PlatformError::APIError(0)),
    };
    unsafe {
      let inputs = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
          mi: MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
          },
        },
      }];

      let res = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
      if res != inputs.len() as u32 {
        return Err(PlatformError::APIError(res as i32));
      }
    }
    Ok(())
  }
  fn clickdown(&self, button: u32) -> Result<(), PlatformError> {
    let flags = match button {
      0 => MOUSEEVENTF_LEFTDOWN,
      1 => MOUSEEVENTF_RIGHTDOWN,
      2 => MOUSEEVENTF_MIDDLEDOWN,
      _ => return Err(PlatformError::APIError(0)),
    };
    unsafe {
      let inputs = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
          mi: MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
          },
        },
      }];

      let res = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
      if res != inputs.len() as u32 {
        return Err(PlatformError::APIError(res as i32));
      }
    }
    Ok(())
  }
  fn moveabsolute(&self, x: f32, y: f32, bp: (i32, i32, i32, i32)) -> Result<(), PlatformError> {
    // basepoint is (x, y, w, h)
    let x = bp.0 + (x * bp.2 as f32) as i32;
    let y = bp.1 + (y * bp.3 as f32) as i32;
    unsafe {
      let res = SetCursorPos(x, y);
      if res.is_err() {
        return Err(PlatformError::APIError(-1));
      }
    }
    Ok(())
  }
  fn wheel(&self, delta_x: f32, delta_y: f32) -> Result<(), PlatformError> {
    let mut inputs = Vec::with_capacity(2);
    let dx_int = delta_x.round() as i32;
    let dy_int = delta_y.round() as i32;

    if dy_int != 0 {
      inputs.push(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
          mi: MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: dy_int as u32,
            dwFlags: MOUSEEVENTF_WHEEL,
            time: 0,
            dwExtraInfo: 0,
          },
        },
      });
    }
    if dx_int != 0 {
      inputs.push(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
          mi: MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: dx_int as u32,
            dwFlags: MOUSEEVENTF_HWHEEL,
            time: 0,
            dwExtraInfo: 0,
          },
        },
      });
    }
    
    if !inputs.is_empty() {
      unsafe {
        let res = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if res != inputs.len() as u32 {
          return Err(PlatformError::APIError(res as i32));
        }
      }
    }
    Ok(())
  }
}

impl KeyboardBackend for WindowsInputInjector {
  fn key_down(&self, key: &Option<&LogicalKey>) -> Result<(), PlatformError> {
    if let Some(LogicalKey::WindowsKey(windows_key)) = *key {
      unsafe {
        let ki = KEYBDINPUT {
          wVk: VIRTUAL_KEY(0),
          wScan: windows_key.scancode as u16,
          dwFlags: if windows_key.extended {
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY
          } else {
            KEYEVENTF_SCANCODE
          },
          time: 0,
          dwExtraInfo: 0,
        };
        let input = INPUT {
          r#type: INPUT_KEYBOARD,
          Anonymous: INPUT_0 { ki },
        };
        let inputs = [input];
        let res = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if res == 0 {
          return Err(PlatformError::APIError(res as i32));
        }
        Ok(())
      }
    } else {
      Err(PlatformError::TypeError)
    }
  }
  fn key_up(&self, key: &Option<&LogicalKey>) -> Result<(), PlatformError> {
    if let Some(LogicalKey::WindowsKey(windows_key)) = *key {
      unsafe {
        let ki = KEYBDINPUT {
          wVk: VIRTUAL_KEY(0),
          wScan: windows_key.scancode as u16,
          dwFlags: if windows_key.extended {
            KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
          } else {
            KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
          },
          time: 0,
          dwExtraInfo: 0,
        };
        let input = INPUT {
          r#type: INPUT_KEYBOARD,
          Anonymous: INPUT_0 { ki },
        };
        let inputs = [input];
        let res = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if res == 0 {
          return Err(PlatformError::APIError(res as i32));
        }
        Ok(())
      }
    } else {
      Err(PlatformError::TypeError)
    }
  }
}

#[async_trait]
impl InputInjectorGeneral for WindowsInputInjector {
  async fn handle_input(&self, _stat: &InputStat, _string: String) -> Result<(), PlatformError> {
    let parts = _string.split(" ").collect::<Vec<&str>>();
    // println!("Received input: {}", _string);
    // println!("Parts: {:?}", parts);
    if parts.len() == 0 {
      return Err(PlatformError::TypeError);
    }
    let cmd = parts[0];
    match cmd {
      "KEY_DOWN" => {
        if parts.len() != 2 {
          return Err(PlatformError::TypeError);
        }
        let key_name = parts[1];
        let key = _stat.keymap.get(key_name).ok_or(PlatformError::TypeError)?;
        self.key_down(&Some(key))
      }
      "KEY_UP" => {
        if parts.len() != 2 {
          return Err(PlatformError::TypeError);
        }
        let key_name = parts[1];
        let key = _stat.keymap.get(key_name).ok_or(PlatformError::TypeError)?;
        self.key_up(&Some(key))
      }
      "MOUSE_CLICK_DOWN" => {
        if parts.len() != 2 {
          return Err(PlatformError::TypeError);
        }
        let button = parts[1].parse::<u32>().map_err(|_e| PlatformError::TypeError)?;
        self.clickdown(button)
      }
      "MOUSE_CLICK_UP" => {
        if parts.len() != 2 {
          return Err(PlatformError::TypeError);
        }
        let button = parts[1].parse::<u32>().map_err(|_e| PlatformError::TypeError)?;
        self.clickup(button)
      }
      "MOUSE_MOVE" => {
        if parts.len() != 3 {
          return Err(PlatformError::TypeError);
        }
        let x = parts[1].parse::<f32>().map_err(|_e| PlatformError::TypeError)?;
        let y = parts[2].parse::<f32>().map_err(|_e| PlatformError::TypeError)?;
        let bp = _stat.screen.get_base_point();
        self.moveabsolute(x, y, bp)
      }
      "MOUSE_WHEEL" => {
        if parts.len() != 3 {
          return Err(PlatformError::TypeError);
        }
        let delta_x = parts[1].parse::<f32>().map_err(|_e| PlatformError::TypeError)?;
        let delta_y = parts[2].parse::<f32>().map_err(|_e| PlatformError::TypeError)?;
        self.wheel(delta_x, delta_y)
      }
      _ => Err(PlatformError::TypeError),
    }
  }
  // fn get_keymap_path(&self) -> String {
  //   String::from("keymaps/windows.toml")
  // }
  fn load_keymap(&self) -> Result<HashMap<String, LogicalKey>, PlatformError> {
    let content = include_str!("../../../../keymaps/windows.toml");
    let config: HashMap<String, WindowsKey> = toml::from_str(content).map_err(|_e| PlatformError::APIError(1))?;
    let keymap = config
      .into_iter()
      .map(|(k, v)| (k, LogicalKey::WindowsKey(v)))
      .collect();
    Ok(keymap)
  }
  fn new() -> Self {
    WindowsInputInjector
  }
}

impl InputInjector for WindowsInputInjector {}
