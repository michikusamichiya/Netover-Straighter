use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:3000".to_string(),
        }
    }
}

fn get_config_path(app: &AppHandle) -> std::path::PathBuf {
    let mut path = app.path().app_config_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> core::result::Result<AppConfig, String> {
    let path = get_config_path(&app);
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
pub fn set_config(app: AppHandle, config: AppConfig) -> core::result::Result<(), String> {
    let path = get_config_path(&app);
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
