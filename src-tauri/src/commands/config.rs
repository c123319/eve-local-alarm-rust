use crate::models::MonitorConfig;
use crate::store::ConfigStore;

/// Save configuration to disk
#[tauri::command]
pub async fn save_config(config: MonitorConfig) -> Result<String, String> {
    let store = ConfigStore::new()
        .map_err(|e| format!("Failed to initialize config store: {}", e))?;

    store.save_config(&config)?;

    Ok("配置已保存".to_string())
}

/// Load configuration from disk
#[tauri::command]
pub async fn load_config() -> Result<MonitorConfig, String> {
    let store = ConfigStore::new()
        .map_err(|e| format!("Failed to initialize config store: {}", e))?;

    let config = store.load_config()?;

    // Return config (will be cloned when used in monitoring)
    Ok(config)
}

/// Get default configuration
#[tauri::command]
pub async fn get_default_config() -> Result<MonitorConfig, String> {
    Ok(ConfigStore::get_default_config())
}

/// Get config file status
#[tauri::command]
pub async fn get_config_status() -> Result<ConfigStatus, String> {
    let store = ConfigStore::new()
        .map_err(|e| format!("Failed to initialize config store: {}", e))?;

    let path = store.config_path().to_string_lossy().to_string();

    let (exists, valid, last_modified) = if store.config_path().exists() {
        let metadata = std::fs::metadata(store.config_path())
            .map_err(|e| format!("Failed to read config metadata: {}", e))?;

        let last_modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let valid = store.load_config().is_ok();
        (true, valid, last_modified)
    } else {
        (false, false, 0)
    };

    Ok(ConfigStatus {
        path,
        exists,
        valid,
        last_modified,
    })
}

/// Config file status
#[derive(serde::Serialize)]
pub struct ConfigStatus {
    pub path: String,
    pub exists: bool,
    pub valid: bool,
    pub last_modified: u64,
}

/// Runtime freeze: Create a frozen copy for monitoring
/// This will be called in Phase 2 when monitoring starts
pub fn create_frozen_config(config: &MonitorConfig) -> MonitorConfig {
    config.clone() // Deep copy via Clone derive
}
