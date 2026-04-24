use serde_json::Value;

/// Save configuration to disk
/// Placeholder for Plan 02 - actual implementation will load/save MonitorConfig
#[tauri::command]
pub async fn save_config(_config: Value) -> Result<(), String> {
    // TODO: Implement config save in Plan 02
    // - Validate config structure
    // - Write to platform-appropriate path via dirs crate
    // - Emit config-saved event
    Ok(())
}

/// Load configuration from disk
/// Placeholder for Plan 02 - actual implementation will load MonitorConfig
#[tauri::command]
pub async fn load_config() -> Result<Value, String> {
    // TODO: Implement config load in Plan 02
    // - Read from platform-appropriate path via dirs crate
    // - Parse and validate JSON
    // - Return MonitorConfig as Value
    Ok(serde_json::json!({}))
}
