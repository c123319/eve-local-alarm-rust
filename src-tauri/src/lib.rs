// Command modules
mod commands;
// Model modules
mod models;
// Store modules
mod store;
// DPI module
mod dpi;
// Capture module
mod capture;

use commands::{save_config, load_config, get_default_config, get_config_status, get_dpi_info, validate_roi_coordinates};

/// Event names for Rust → Frontend communication
pub mod events {
    pub const CONFIG_SAVED: &str = "config-saved";
    pub const CONFIG_LOADED: &str = "config-loaded";
    pub const ERROR: &str = "error";
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            get_default_config,
            get_config_status,
            get_dpi_info,
            validate_roi_coordinates,
        ])
        .setup(|_app| {
            // Event channel setup for future Phase 2+ use
            // Example pattern: _app.emit(events::CONFIG_SAVED, payload)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
