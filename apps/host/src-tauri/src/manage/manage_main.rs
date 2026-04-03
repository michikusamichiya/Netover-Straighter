use crate::pairing::pairing_main::StoredKey;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
pub struct EditOptions {
    pub avail: Option<bool>,
    pub id: Option<String>,
}

fn mask_prefix(s: &str, visible: usize) -> String {
    let prefix: String = s.chars().take(visible).collect();
    format!("{}...", prefix)
}

#[tauri::command]
pub async fn loadkeys(app: AppHandle, is_key_id_only: bool) -> Result<serde_json::Value, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    let keys_file = app_data_dir.join("pairing_keys.json");

    if !keys_file.exists() {
        if is_key_id_only {
            return Ok(serde_json::json!([]));
        } else {
            return Ok(serde_json::json!({}));
        }
    }

    let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
    let mut keys: HashMap<String, StoredKey> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse keys: {}", e))?;

    if is_key_id_only {
        let id_list: Vec<String> = keys.keys().cloned().collect();
        Ok(serde_json::to_value(id_list).unwrap())
    } else {
        for (id, key) in keys.iter_mut() {
            if let Ok(entry) = keyring::Entry::new("netover-bloodway", id) {
                if let Ok(password) = entry.get_password() {
                    key.body = mask_prefix(&password, 4);
                }
            }
        }
        Ok(serde_json::to_value(keys).unwrap())
    }
}

#[tauri::command]
pub async fn deletekey(app: AppHandle, id: String) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    let keys_file = app_data_dir.join("pairing_keys.json");

    if !keys_file.exists() {
        return Err("No keys file found".to_string());
    }

    let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
    let mut keys: HashMap<String, StoredKey> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse keys: {}", e))?;

    if keys.remove(&id).is_none() {
        return Err("No key found for the given ID".to_string());
    }

    if let Ok(entry) = keyring::Entry::new("netover-bloodway", &id) {
        let _ = entry.delete_password();
    }

    let new_content = serde_json::to_string_pretty(&keys).map_err(|e| format!("Failed to serialize keys: {}", e))?;
    fs::write(&keys_file, new_content).map_err(|e| format!("Failed to write keys: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn editkey(app: AppHandle, id: String, opt: EditOptions) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    let keys_file = app_data_dir.join("pairing_keys.json");

    if !keys_file.exists() {
        return Err("No keys file found".to_string());
    }

    let content = fs::read_to_string(&keys_file).map_err(|e| format!("Failed to read keys: {}", e))?;
    let mut keys: HashMap<String, StoredKey> = serde_json::from_str(&content).map_err(|e| format!("Failed to parse keys: {}", e))?;

    let mut key_data = keys.remove(&id).ok_or_else(|| "No key found for the given ID".to_string())?;

    if let Some(avail) = opt.avail {
        key_data.available = avail;
    }

    let final_id = if let Some(new_id) = opt.id {
        if !regex::Regex::new(r"^[A-Z]{6}$").unwrap().is_match(&new_id) {
            keys.insert(id, key_data); // restore
            return Err("ID must be 6 uppercase A-Z letters.".to_string());
        }
        if id != new_id {
            if let Ok(old_entry) = keyring::Entry::new("netover-bloodway", &id) {
                if let Ok(password) = old_entry.get_password() {
                    let _ = old_entry.delete_password();
                    if let Ok(new_entry) = keyring::Entry::new("netover-bloodway", &new_id) {
                        let _ = new_entry.set_password(&password);
                    }
                }
            }
        }
        new_id
    } else {
        id
    };

    keys.insert(final_id, key_data);

    let new_content = serde_json::to_string_pretty(&keys).map_err(|e| format!("Failed to serialize keys: {}", e))?;
    fs::write(&keys_file, new_content).map_err(|e| format!("Failed to write keys: {}", e))?;

    Ok(())
}
