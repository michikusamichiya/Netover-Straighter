// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appstate;
mod pairing;
mod manage;
mod ws;
mod crypto;
mod emitter;
mod launch;
use tokio::sync::Mutex;

use crate::launch::InputInjector;
use crate::launch::ScreenOutputer::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let injector = InputInjector::create_injector();
  let cway = Capture::create_capture_way();

  tauri::Builder::default()
    .manage(appstate::AppState {
      pairing: Mutex::new(None).into(),
      launching: Mutex::new(None).into(),
      input_trait: injector,
      input_stat: Mutex::new(None).into(),
      capture_trait: cway,
      capture_stat: Mutex::new(None).into(),
    })
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      pairing::pairing_main::start_pairing,
      pairing::pairing_main::end_pairing,
      pairing::pairing_main::accept,
      pairing::pairing_main::deny,
      pairing::pairing_main::cancel,
      pairing::pairing_main::save_key,

      manage::manage_main::loadkeys,
      manage::manage_main::deletekey,
      manage::manage_main::editkey,

      launch::launching_main::launch,
      launch::launching_main::end_launching,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
