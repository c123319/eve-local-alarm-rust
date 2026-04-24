use serde::{Serialize, Deserialize};

/// Main monitoring configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MonitorConfig {
    pub targets: Vec<TargetConfig>,
    pub rois: Vec<RoiConfig>,
    pub alert: AlertConfig,
    pub debug: DebugConfig,
}

/// Configuration for a target window (WGC mode)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TargetConfig {
    pub id: String,
    pub window_title: String,
    pub capture_mode: CaptureMode,
}

/// Configuration for a region of interest (ROI)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RoiConfig {
    pub id: String,
    pub name: String,
    pub capture_mode: CaptureMode,
    pub region: Rect,
    pub color_rules: Vec<ColorMatchConfig>,
    pub debounce_ms: u64,
    pub dpi_invalidation_flags: DpiInvalidationFlags,
}

/// Alert configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AlertConfig {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub sound_path: String,
    pub toast_enabled: bool,
    pub cooldown_ms: u64,
}

/// Debug configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DebugConfig {
    pub enabled: bool,
    pub dump_hsv_masks: bool,
    pub dump_overlays: bool,
    pub debug_dir: String,
}

/// Capture mode enumeration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CaptureMode {
    MSS,
    WGC,
}

impl Default for CaptureMode {
    fn default() -> Self { CaptureMode::MSS }
}

/// Rectangle region (in physical pixels)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Color matching rule configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ColorMatchConfig {
    pub name: String,
    pub hsv_lower: [u32; 3],
    pub hsv_upper: [u32; 3],
    pub min_pixels: u32,
    pub min_ratio: f64,
}

/// DPI invalidation flags for ROI regions
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DpiInvalidationFlags {
    pub invalid: bool,
    pub last_dpi_scale: f64,
    pub last_display_id: Option<String>,
}
