#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod elevate;
mod node;
mod pkg;
mod service;
mod settings;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_node,
            commands::provision_node,
            commands::clean_data,
            commands::list_versions,
            commands::start_service,
            commands::stop_service,
            commands::status,
            commands::get_logs,
            commands::load_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
