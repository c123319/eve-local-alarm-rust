# Phase 3: HSV Detection Engine - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 6 (4 new, 2 modified)
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/detection/mod.rs` | config | -- | `src-tauri/src/capture/mod.rs` | exact |
| `src-tauri/src/detection/hsv.rs` | utility | transform | `src-tauri/src/dpi/contract.rs` | role-match |
| `src-tauri/src/detection/engine.rs` | service | streaming | `src-tauri/src/capture/mss.rs` | exact |
| `src-tauri/src/detection/validation.rs` | utility | request-response | `src-tauri/src/capture/mss.rs` (validate_* helpers) | exact |
| `src-tauri/src/lib.rs` | config | -- | `src-tauri/src/lib.rs` (existing) | self |
| `src-tauri/src/commands/monitoring.rs` | controller | request-response | `src-tauri/src/commands/monitoring.rs` (existing) | self |

## Pattern Assignments

### `src-tauri/src/detection/mod.rs` (config, module declaration)

**Analog:** `src-tauri/src/capture/mod.rs`

**Full file pattern** (lines 1-3):
```rust
pub mod mss;

pub use mss::{CapturedFrame, MonitoringSnapshot, MonitoringStatus, MssCaptureWorker};
```

**Apply as:** Create `detection/mod.rs` with the same two-section structure: `pub mod` declarations for each sub-module, followed by `pub use` re-exports for the public API surface.

```rust
pub mod hsv;
pub mod engine;
pub mod validation;

pub use engine::{DetectionEngine, DetectionResult, RuleMatchResult};
pub use hsv::rgba_pixel_to_hsv;
pub use validation::validate_color_match_config;
```

---

### `src-tauri/src/detection/hsv.rs` (utility, transform)

**Analog:** `src-tauri/src/dpi/contract.rs`

Both are pure-function utility modules: no structs, no state, just conversion helpers and predicates. The DPI module exports `to_physical`, `to_display`, `check_dpi_invalidation` as free functions. The HSV module exports `rgba_pixel_to_hsv` and pixel-counting helpers as free functions.

**Imports pattern** (from `dpi/contract.rs` lines 1-2):
```rust
use serde::{Serialize, Deserialize};
```

For `hsv.rs`, the import block will reference the config model:
```rust
use crate::models::ColorMatchConfig;
```

**Free function + inline helper pattern** (from `dpi/contract.rs` lines 27-32):
```rust
/// 将显示坐标转换为物理坐标
pub fn to_physical(display: DisplayCoord, scale: f64) -> PhysicalCoord {
    PhysicalCoord {
        x: (display.x as f64 * scale).round() as i32,
        y: (display.y as f64 * scale).round() as i32,
    }
}
```

Apply as: `rgba_pixel_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8)` with `#[inline]` attribute, and `count_matching_pixels(rgba: &[u8], rule: &ColorMatchConfig) -> usize` as a public function.

**Test pattern** (from `dpi/contract.rs` lines 86-97):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_physical() {
        let display = DisplayCoord { x: 100, y: 100 };
        let physical = to_physical(display, 1.5);
        assert_eq!(physical.x, 150);
        assert_eq!(physical.y, 150);
    }
}
```

Apply as: Unit tests with known RGB-to-HSV pairs. Pure red (255,0,0) should map to H near 0, S=255, V=255. Test the OpenCV half-range convention explicitly.

---

### `src-tauri/src/detection/engine.rs` (service, streaming)

**Analog:** `src-tauri/src/capture/mss.rs`

This is the most critical analog. `MssCaptureWorker` establishes every pattern the detection engine needs: struct with `Arc<AtomicBool>` cancellation + `JoinHandle` + `Arc<Mutex<Option<T>>>` latest-result slot, `start()/stop()/get_latest_result()` lifecycle, and a threaded loop.

**Imports pattern** (from `capture/mss.rs` lines 1-6):
```rust
use crate::models::Rect;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
```

For `engine.rs`, imports will be:
```rust
use crate::capture::CapturedFrame;
use crate::models::ColorMatchConfig;
use super::hsv::count_matching_pixels;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
```

**Data struct pattern** (from `capture/mss.rs` lines 9-23):
```rust
/// 捕获的帧数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapturedFrame {
    pub roi_id: String,
    pub region: Rect,
    pub captured_at_ms: u128,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
```

Apply as: `DetectionResult` and `RuleMatchResult` structs with `#[derive(Clone, Debug, Serialize, Deserialize)]`. Chinese doc comments. All fields `pub`.

**Worker struct pattern** (from `capture/mss.rs` lines 56-67):
```rust
pub struct MssCaptureWorker {
    cancellation: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    roi_id: String,
    region: Rect,
    capture_fps: u32,
}
```

Apply as: `DetectionWorker` (or inline engine, planner's choice) with:
```rust
pub struct DetectionWorker {
    cancellation: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    latest_result: Arc<Mutex<Option<DetectionResult>>>,
    color_rules: Vec<ColorMatchConfig>,
    frame_source: Arc<Mutex<Option<CapturedFrame>>>,
}
```

**Constructor pattern** (from `capture/mss.rs` lines 69-80):
```rust
impl MssCaptureWorker {
    pub fn new(roi_id: String, region: Rect, capture_fps: u32) -> Self {
        Self {
            cancellation: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            latest_frame: Arc::new(Mutex::new(None)),
            roi_id,
            region,
            capture_fps,
        }
    }
```

**Start lifecycle** (from `capture/mss.rs` lines 83-112):
```rust
    pub fn start(&mut self) -> Result<(), String> {
        // ... validation ...

        if self.thread_handle.is_some() {
            return Err("监控已在运行中".to_string());
        }

        self.cancellation.store(false, Ordering::SeqCst);

        let cancellation = Arc::clone(&self.cancellation);
        let latest_frame = Arc::clone(&self.latest_frame);
        // ... clone other fields ...

        let handle = thread::spawn(move || {
            Self::capture_loop(cancellation, latest_frame, roi_id, region, capture_fps);
        });

        self.thread_handle = Some(handle);
        Ok(())
    }
```

**Stop lifecycle** (from `capture/mss.rs` lines 114-127):
```rust
    pub fn stop(&mut self) -> Result<(), String> {
        self.cancellation.store(true, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|e| {
                format!("等待捕获线程结束失败: {:?}", e)
            })?;
        }

        Ok(())
    }
```

**Drop guard pattern** (from `capture/mss.rs` lines 216-223):
```rust
impl Drop for MssCaptureWorker {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            let _ = self.stop();
        }
    }
}
```

**Mutex poison recovery** (from `capture/mss.rs` lines 130-135):
```rust
    pub fn get_latest_frame(&self) -> Option<CapturedFrame> {
        match self.latest_frame.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
```

Apply identically for `get_latest_result()` on `DetectionResult`.

**Thread loop pattern** (from `capture/mss.rs` lines 138-178):
```rust
    fn capture_loop(
        cancellation: Arc<AtomicBool>,
        latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
        roi_id: String,
        region: Rect,
        capture_fps: u32,
    ) {
        let frame_duration = Duration::from_millis(1000 / capture_fps as u64);

        loop {
            if cancellation.load(Ordering::SeqCst) {
                break;
            }

            let start_time = Instant::now();

            // ... do work ...

            let elapsed = start_time.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    }
```

Apply as: detection loop that reads from `frame_source`, calls `evaluate_frame()`, writes to `latest_result`. Same `AtomicBool` break check, same `Instant` timing for cadence control. The detection loop should sleep briefly (or busy-wait with a small duration) when no new frame is available.

**now_millis helper** (from `capture/mss.rs` lines 244-250):
```rust
pub fn now_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
```

Reuse this from `capture::mss::now_millis` or duplicate it in the detection module.

**Test pattern** (from `capture/mss.rs` lines 252-321):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_positive_roi_dimensions() {
        // ... construct input, assert result ...
    }
}
```

Apply as: Construct synthetic `CapturedFrame` with known RGBA data (e.g., all-red pixels, all-black pixels, mixed). Verify `DetectionResult.detected`, `rule_results[].pixel_count`, `rule_results[].ratio`.

---

### `src-tauri/src/detection/validation.rs` (utility, request-response)

**Analog:** `src-tauri/src/capture/mss.rs` (validate_capture_fps, validate_roi)

These are free functions that return `Result<T, String>` with Chinese error messages.

**Validation function pattern** (from `capture/mss.rs` lines 226-241):
```rust
/// 验证捕获帧率
pub fn validate_capture_fps(fps: u32) -> Result<u32, String> {
    if (1..=30).contains(&fps) {
        Ok(fps)
    } else {
        Err("捕获帧率必须在 1 到 30 FPS 之间".to_string())
    }
}

/// 验证 ROI 区域
pub fn validate_roi(region: &Rect) -> Result<(), String> {
    if region.width == 0 || region.height == 0 {
        Err("ROI 宽度和高度必须大于 0".to_string())
    } else {
        Ok(())
    }
}
```

Apply as:
```rust
/// 验证颜色匹配配置
pub fn validate_color_match_config(config: &ColorMatchConfig) -> Result<(), String> {
    if config.min_pixels == 0 {
        return Err("最小像素数必须大于 0".to_string());
    }
    if config.min_ratio <= 0.0 || config.min_ratio > 1.0 {
        return Err("最小像素比例必须在 (0.0, 1.0] 范围内".to_string());
    }
    for ch in 0..3 {
        if config.hsv_lower[ch] > config.hsv_upper[ch] {
            return Err(format!(
                "HSV 下界不能大于上界 (通道 {}): {} > {}",
                ch, config.hsv_lower[ch], config.hsv_upper[ch]
            ));
        }
    }
    Ok(())
}
```

**Test pattern** (from `capture/mss.rs` lines 294-309):
```rust
    #[test]
    fn validates_fps_range() {
        assert!(validate_capture_fps(1).is_ok());
        assert!(validate_capture_fps(5).is_ok());
        assert!(validate_capture_fps(30).is_ok());
    }

    #[test]
    fn rejects_invalid_fps_range() {
        let result = validate_capture_fps(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("捕获帧率必须在 1 到 30 FPS 之间"));
    }
```

Apply as: Test valid config passes, zero min_pixels rejected, out-of-range min_ratio rejected, inverted HSV bounds rejected. All error assertions check Chinese text substrings.

---

### `src-tauri/src/lib.rs` (config, module registration)

**Analog:** Self (existing file, modification only)

**Current module declarations** (lines 1-10):
```rust
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
```

**Change required:** Add detection module declaration:
```rust
// Detection module
mod detection;
```

**Current event constants** (lines 19-27):
```rust
pub mod events {
    pub const CONFIG_SAVED: &str = "config-saved";
    pub const CONFIG_LOADED: &str = "config-loaded";
    pub const ERROR: &str = "error";
    pub const MONITORING_STATUS: &str = "monitoring-status";
    pub const MONITORING_ERROR: &str = "monitoring-error";
}
```

**Change required (optional per D-11):** If emitting detection events, add:
```rust
    /// Emitted when a detection result is produced.
    pub const DETECTION_RESULT: &str = "detection-result";
```

---

### `src-tauri/src/commands/monitoring.rs` (controller, request-response)

**Analog:** Self (existing file, modification to wire detection)

**Current Tauri command pattern** (lines 91-95):
```rust
#[tauri::command]
pub async fn start_monitoring(
    config: MonitorConfig,
    state: tauri::State<'_, MonitoringController>,
    app: tauri::AppHandle,
) -> Result<MonitoringSnapshot, String> {
```

**Event emission pattern** (lines 104, 130-138):
```rust
let _ = app.emit(crate::events::MONITORING_ERROR, &error);
// ...
let _ = app.emit(
    crate::events::MONITORING_STATUS,
    &MonitoringSnapshot { ... },
);
```

**Current inner state struct** (lines 16-22):
```rust
struct MonitoringControllerInner {
    status: MonitoringStatus,
    last_error: Option<String>,
    worker: Option<MssCaptureWorker>,
    capture_fps: u32,
}
```

**Change required:** Add detection worker or detection engine field to `MonitoringControllerInner`. If detection is inline (runs on capture thread), no new thread needed -- just call `DetectionEngine::evaluate_frame()` inside the capture loop. If separate thread, add a `detection_worker: Option<DetectionWorker>` field.

**Event emission for detection results:** Follow the `app.emit()` pattern to emit `crate::events::DETECTION_RESULT` with `&DetectionResult`.

**Error handling pattern** (lines 158-174):
```rust
        Err(e) => {
            let snapshot = {
                let mut inner = state
                    .inner
                    .lock()
                    .map_err(|_| "内部状态锁定失败".to_string())?;
                inner.status = MonitoringStatus::Error;
                inner.last_error = Some(e.clone());
                inner.to_snapshot()
            };
            let _ = app.emit(crate::events::MONITORING_STATUS, &snapshot);
            let _ = app.emit(crate::events::MONITORING_ERROR, &e);
            Err(e)
        }
```

Apply same error-reporting pattern for any detection startup failures.

---

## Shared Patterns

### Thread Lifecycle (AtomicBool + JoinHandle + Arc<Mutex<Option<T>>>)
**Source:** `src-tauri/src/capture/mss.rs` lines 56-127, 216-223
**Apply to:** `detection/engine.rs` DetectionWorker (if separate thread model chosen)

```rust
pub struct Worker {
    cancellation: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    latest_result: Arc<Mutex<Option<T>>>,
}
// Constructor initializes AtomicBool(false), None handle, Mutex(None)
// start(): validate -> check not running -> reset cancellation -> clone Arcs -> spawn -> store handle
// stop(): store true on cancellation -> take handle -> join()
// Drop: if handle.is_some() { let _ = self.stop(); }
```

### Mutex Poison Recovery
**Source:** `src-tauri/src/capture/mss.rs` lines 130-135, 159-162
**Apply to:** All `Arc<Mutex<Option<T>>>` accesses in detection module

```rust
match some_mutex.lock() {
    Ok(guard) => guard.clone(),
    Err(poisoned) => poisoned.into_inner().clone(),
}
```

### Validation Helpers (Chinese messages, Result<(), String>)
**Source:** `src-tauri/src/capture/mss.rs` lines 226-241
**Apply to:** `detection/validation.rs`

```rust
pub fn validate_xxx(value: T) -> Result<(), String> {
    if /* valid */ {
        Ok(())
    } else {
        Err("中文错误消息".to_string())
    }
}
```

### Tauri Event Emission
**Source:** `src-tauri/src/commands/monitoring.rs` lines 104, 130-138
**Apply to:** Detection result emission in monitoring.rs

```rust
use tauri::Emitter;
// ...
let _ = app.emit(crate::events::EVENT_NAME, &serializable_data);
```

### Serde Derive on Data Structs
**Source:** `src-tauri/src/capture/mss.rs` lines 9, 26, 43
**Apply to:** `DetectionResult`, `RuleMatchResult` in `detection/engine.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructName {
    /// 中文文档注释
    pub field: Type,
}
```

### now_millis Timestamp Helper
**Source:** `src-tauri/src/capture/mss.rs` lines 244-250
**Apply to:** Detection result timestamps

```rust
pub fn now_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
```

**Decision:** Import from `crate::capture::mss::now_millis` to avoid duplication, or re-export from capture module.

## No Analog Found

All files have close analogs in the existing codebase. No files require reliance on RESEARCH.md code examples alone.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| -- | -- | -- | All files have analogs |

## Metadata

**Analog search scope:** `src-tauri/src/` (all 14 existing .rs files)
**Files scanned:** 14
**Pattern extraction date:** 2026-04-28
