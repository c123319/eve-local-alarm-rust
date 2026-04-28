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
// Detection module
mod detection;

use commands::{
    get_config_status, get_default_config, get_dpi_info, get_monitoring_status, load_config,
    save_config, start_monitoring, stop_monitoring, validate_roi_coordinates,
};
use commands::monitoring::{evaluate_latest_frame, MonitoringController};

/// Event names for Rust → Frontend communication.
pub mod events {
    pub const CONFIG_SAVED: &str = "config-saved";
    pub const CONFIG_LOADED: &str = "config-loaded";
    pub const ERROR: &str = "error";
    /// Emitted whenever the monitoring status changes.
    pub const MONITORING_STATUS: &str = "monitoring-status";
    /// Emitted when a monitoring error occurs.
    pub const MONITORING_ERROR: &str = "monitoring-error";
    /// 检测结果事件（供 Phase 4 告警消费）
    pub const DETECTION_RESULT: &str = "detection-result";
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MonitoringController::default())
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            get_default_config,
            get_config_status,
            get_dpi_info,
            validate_roi_coordinates,
            start_monitoring,
            stop_monitoring,
            get_monitoring_status,
            evaluate_latest_frame,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
