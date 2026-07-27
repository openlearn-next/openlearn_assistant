use crate::node::NodeInfo;
use crate::pkg;
use crate::service::{self, Status};
use crate::settings::Settings;

#[tauri::command]
pub fn detect_node() -> NodeInfo {
    crate::node::detect_node()
}

#[tauri::command]
pub fn provision_node() -> Result<(), String> {
    crate::node::provision_node()
}

#[tauri::command]
pub fn install_pkg() -> Result<(), String> {
    pkg::install_pkg()
}

#[tauri::command]
pub fn uninstall_pkg(keep_data: bool) -> Result<(), String> {
    pkg::uninstall_pkg(keep_data)
}

#[tauri::command]
pub fn upgrade_pkg() -> Result<(), String> {
    pkg::upgrade_pkg()
}

#[tauri::command]
pub fn start_service() -> Result<(), String> {
    service::start_service()
}

#[tauri::command]
pub fn stop_service() -> Result<(), String> {
    service::stop_service()
}

#[tauri::command]
pub fn status() -> Status {
    service::status()
}

#[tauri::command]
pub fn get_logs(tail: u32) -> String {
    service::get_logs(tail)
}

#[tauri::command]
pub fn load_settings() -> Settings {
    crate::settings::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<(), String> {
    crate::settings::save_settings(&settings).map_err(|e| e.to_string())
}
