use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub db_path: String,
    pub gemini_api_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            port: 9000,
            db_path: default_db_path(),
            gemini_api_key: String::new(),
        }
    }
}

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn app_dir() -> PathBuf {
    home_dir().join(".openlearn-next-gui")
}

pub fn settings_path() -> PathBuf {
    app_dir().join("settings.json")
}

pub fn log_path() -> PathBuf {
    app_dir().join("openlearn-next.log")
}

pub fn pid_path() -> PathBuf {
    app_dir().join("openlearn-next.pid")
}

pub fn default_db_path() -> String {
    home_dir()
        .join("openlearn-next")
        .join("data.db")
        .to_string_lossy()
        .into_owned()
}

pub fn load_settings() -> Settings {
    let p = settings_path();
    if let Ok(s) = fs::read_to_string(p) {
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        Settings::default()
    }
}

pub fn save_settings(s: &Settings) -> std::io::Result<()> {
    fs::create_dir_all(app_dir())?;
    fs::write(settings_path(), serde_json::to_string_pretty(s).unwrap())
}
