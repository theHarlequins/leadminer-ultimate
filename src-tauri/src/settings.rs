use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api_key: String,
    pub model_id: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model_id: "google/gemini-2.0-flash-exp:free".to_string(), // Default sensible model
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    settings: Mutex<Settings>,
}

impl SettingsStore {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let path = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir")
            .join("settings.json");

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let settings = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Settings::default(),
            }
        } else {
            Settings::default()
        };

        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn save(&self, new_settings: Settings) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        fs::write(&self.path, json)
            .map_err(|e| format!("Write error: {}", e))?;

        *self.settings.lock().unwrap() = new_settings;
        Ok(())
    }
}
