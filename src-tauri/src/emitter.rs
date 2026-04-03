use tauri::{AppHandle, Emitter};
use serde::Serialize;

pub fn emit_safer<T, F>(
    app: &AppHandle,
    event: &str,
    data: T,
    on_error: F,
)
where
    T: Serialize + Clone,
    F: Fn(String),
{
    if let Err(e) = app.emit(event, data) {
        on_error(e.to_string());
    }
}