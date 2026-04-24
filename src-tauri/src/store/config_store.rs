use crate::models::MonitorConfig;
use std::path::PathBuf;
use dirs::config_dir;

/// Config store for saving and loading MonitorConfig to/from disk
pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    /// Create a new ConfigStore with platform-appropriate config path
    pub fn new() -> Result<Self, String> {
        let config_dir = config_dir()
            .ok_or("Failed to get config directory")?;
        let app_config_dir = config_dir.join("eve-local-alarm");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_path = app_config_dir.join("config.json");

        Ok(ConfigStore { config_path })
    }

    /// Save configuration to disk
    pub fn save_config(&self, config: &MonitorConfig) -> Result<(), String> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from disk
    pub fn load_config(&self) -> Result<MonitorConfig, String> {
        if !self.config_path.exists() {
            return Err("Config file not found".to_string());
        }

        let json = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: MonitorConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    /// Get the config file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get default configuration
    pub fn get_default_config() -> MonitorConfig {
        MonitorConfig::default()
    }
}
