# Architecture Research — EVE Local Alert v1.0

## System Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Tauri v2 App                       │
│                                                      │
│  ┌──────────────────┐    ┌─────────────────────┐    │
│  │  React Frontend   │◄──►│  Rust Backend        │    │
│  │  (TypeScript)     │    │                      │    │
│  │                    │    │  ┌────────────────┐  │    │
│  │  - Main Window    │    │  │ Capture Manager│  │    │
│  │  - ROI Selector   │    │  │ (WGC + MSS)    │  │    │
│  │  - Config Panel   │    │  └───────┬────────┘  │    │
│  │  - Alert Popup    │    │          │ frames     │    │
│  │  - System Tray    │    │  ┌───────▼────────┐  │    │
│  │                    │    │  │ Detection Pipe │  │    │
│  │                    │    │  │ (HSV match)    │  │    │
│  │                    │    │  └───────┬────────┘  │    │
│  │                    │    │          │ results    │    │
│  │                    │    │  ┌───────▼────────┐  │    │
│  │                    │    │  │ Alert Manager  │  │    │
│  │                    │    │  │ (popup/sound/  │  │    │
│  │                    │    │  │  toast)        │  │    │
│  └──────────────────┘    │  └────────────────┘  │    │
│                           │                      │    │
│                           │  ┌────────────────┐  │    │
│                           │  │ Config Store   │  │    │
│                           │  │ (JSON + freeze)│  │    │
│                           │  └────────────────┘  │    │
│                           └──────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

## Data Flow

### Capture → Detect → Alert Pipeline

```
1. User configures ROI and color rules via React UI
2. React calls Tauri command: invoke("start_monitoring", config)
3. Rust spawns capture threads:
   - MSS: xcap captures desktop region on timer
   - WGC: windows-capture streams frames per EVE window
4. Each frame → Detection Pipeline:
   a. Convert to HSV (opencv cvtColor)
   b. Apply inRange with configured HSV bounds
   c. Count matching pixels (countNonZero)
   d. Check against min_pixels and min_ratio thresholds
5. If threshold exceeded and cooldown expired → emit alert event
6. Alert Manager fires:
   - Tauri event → React shows popup
   - rodio plays sound in background thread
   - tauri-plugin-notification shows Toast
7. Debug mode: dump HSV mask + overlay to disk
```

### Tauri v2 Command Pattern

```rust
// Rust backend — Tauri commands
#[tauri::command]
fn start_monitoring(config: MonitorConfig, app: AppHandle) -> Result<(), String> { ... }

#[tauri::command]
fn stop_monitoring() -> Result<(), String> { ... }

#[tauri::command]
fn get_eve_windows() -> Result<Vec<WindowInfo>, String> { ... }

#[tauri::command]
fn save_config(config: MonitorConfig) -> Result<(), String> { ... }

#[tauri::command]
fn load_config() -> Result<MonitorConfig, String> { ... }

// Rust → Frontend events
app.emit("detection-result", payload)?;
app.emit("alert", alert_payload)?;
app.emit("frame-preview", frame_data)?;
```

```typescript
// React frontend
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Start monitoring
await invoke('start_monitoring', { config });

// Listen for alerts
listen('alert', (event) => { showAlertPopup(event.payload); });
listen('detection-result', (event) => { updateOverlay(event.payload); });
```

## Key Architecture Decisions

### 1. Threading Model

- **Main thread**: Tauri event loop, React webview
- **Capture threads**: One per WGC window, one for MSS region. Use `tokio` async runtime or `std::thread` for capture loops.
- **Detection**: Runs in capture thread after frame capture (synchronous, low latency)
- **Alert dispatch**: Spawn short-lived threads for sound playback (non-blocking)

**Recommended:** Use `tokio` for async coordination, `std::thread` for blocking capture loops with channel-based frame passing.

### 2. Frame Transfer (Rust → Frontend)

For ROI selector preview only (not for detection — that stays in Rust):

- Option A: Encode frame as base64 PNG, emit as Tauri event payload (simple, works for preview)
- Option B: Use shared memory or Tauri asset protocol (faster, more complex)

**Recommendation:** Option A for v1.0. Preview frame rate is low (~5-10 FPS), base64 overhead is acceptable.

### 3. Config Model (Rust structs)

```rust
#[derive(Serialize, Deserialize, Clone)]
struct MonitorConfig {
    targets: Vec<TargetConfig>,    // WGC windows
    rois: Vec<RoiConfig>,          // Capture regions
    alert: AlertConfig,            // Alert settings
    debug: DebugConfig,            // Debug mode settings
}

#[derive(Serialize, Deserialize, Clone)]
struct RoiConfig {
    id: String,
    capture_mode: CaptureMode,     // WGC or MSS
    region: Rect,                  // x, y, width, height
    color_rules: Vec<ColorMatchConfig>,
    debounce_ms: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct ColorMatchConfig {
    name: String,
    hsv_lower: [u32; 3],           // H, S, V min
    hsv_upper: [u32; 3],           // H, S, V max
    min_pixels: u32,
    min_ratio: f64,
}
```

### 4. State Management

- **Rust side**: `Arc<Mutex<>>` or `Arc<RwLock<>>` for shared state (config, monitoring status)
- **React side**: Context or Zustand for UI state (current view, alert history, config form state)
- **Communication**: Tauri commands (request-response) + Tauri events (push notifications)

### 5. Multi-Window Capture Architecture

```
CaptureManager
├── MSS Capture Thread (single ROI, xcap)
│   └── Detect → Alert pipeline
├── WGC Capture Thread #1 (EVE Window A)
│   └── Detect → Alert pipeline
└── WGC Capture Thread #2 (EVE Window B)
    └── Detect → Alert pipeline
```

Each capture thread is independent with its own ROI config and detection pipeline. Alerts are funneled through a single Alert Manager that handles cooldown deduplication.

## Suggested Build Order

1. **Project scaffold**: Tauri v2 + React + TS, basic window
2. **Config model**: Rust structs + JSON serialization + Tauri commands
3. **MSS capture**: xcap desktop region capture, prove pipeline works
4. **HSV detection**: OpenCV integration, color matching on captured frames
5. **Alerts**: Sound + Toast + Popup
6. **ROI selector**: React interactive UI with preview
7. **WGC capture**: Multi-window support via windows-capture
8. **System tray**: Tauri built-in tray
9. **Debug mode**: HSV mask dumps
10. **Polish**: Error handling, edge cases, Chinese UI text
