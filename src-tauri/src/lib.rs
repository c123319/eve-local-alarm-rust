// Command modules
mod commands;

use commands::{save_config, load_config};

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
        ])
        .setup(|_app| {
            // Event channel setup for future Phase 2+ use
            // Example pattern: _app.emit(events::CONFIG_SAVED, payload)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
