use serde::{Deserialize, Serialize};

/// Main monitoring configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MonitorConfig {
    pub capture_fps: u32,
    pub targets: Vec<TargetConfig>,
    pub rois: Vec<RoiConfig>,
    pub alert: AlertConfig,
    pub debug: DebugConfig,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            capture_fps: 5,
            targets: Vec::new(),
            rois: Vec::new(),
            alert: AlertConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

/// Configuration for a target window (WGC mode)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct TargetConfig {
    pub id: String,
    pub window_title: String,
    pub capture_mode: CaptureMode,
}

/// Configuration for a region of interest (ROI)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RoiConfig {
    pub id: String,
    pub name: String,
    pub capture_mode: CaptureMode,
    pub region: Rect,
    pub color_rules: Vec<ColorMatchConfig>,
    pub debounce_ms: u64,
    pub dpi_invalidation_flags: DpiInvalidationFlags,
}

impl Default for RoiConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "本地列表区域".to_string(),
            capture_mode: CaptureMode::MSS,
            region: Rect::default(),
            color_rules: vec![ColorMatchConfig::default_hostile_marker()],
            debounce_ms: 1_500,
            dpi_invalidation_flags: DpiInvalidationFlags::default(),
        }
    }
}

/// Alert configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AlertConfig {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub sound_path: String,
    pub toast_enabled: bool,
    pub cooldown_ms: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            sound_path: String::new(),
            toast_enabled: true,
            cooldown_ms: 3_000,
        }
    }
}

/// Debug configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct DebugConfig {
    pub enabled: bool,
    pub dump_hsv_masks: bool,
    pub dump_overlays: bool,
    pub debug_dir: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dump_hsv_masks: false,
            dump_overlays: false,
            debug_dir: "debug".to_string(),
        }
    }
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
#[serde(default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Color matching rule configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ColorMatchConfig {
    pub name: String,
    pub hsv_lower: [u32; 3],
    pub hsv_upper: [u32; 3],
    pub min_pixels: u32,
    pub min_ratio: f64,
}

impl ColorMatchConfig {
    pub fn default_hostile_marker() -> Self {
        Self {
            name: "敌对标记".to_string(),
            hsv_lower: [0, 120, 120],
            hsv_upper: [15, 255, 255],
            min_pixels: 12,
            min_ratio: 0.02,
        }
    }
}

impl Default for ColorMatchConfig {
    fn default() -> Self {
        Self::default_hostile_marker()
    }
}

/// DPI invalidation flags for ROI regions
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct DpiInvalidationFlags {
    pub invalid: bool,
    pub last_dpi_scale: f64,
    pub last_display_id: Option<String>,
}
