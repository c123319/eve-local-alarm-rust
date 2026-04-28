//! Monitoring lifecycle commands: start, stop, and status query.
//!
//! Manages a single MSS capture worker behind a `Mutex`-guarded state.
//! Emits `monitoring-status` and `monitoring-error` events to the frontend.

use std::sync::Mutex;

use tauri::Emitter;

use crate::capture::{MssCaptureWorker, MonitoringSnapshot, MonitoringStatus};
use crate::commands::config::create_frozen_config;
use crate::detection::validation::validate_color_match_config;
use crate::detection::{DetectionEngine, DetectionResult};
use crate::models::{CaptureMode, MonitorConfig, RoiConfig};

// ─── Inner mutable state ─────────────────────────────────────────────────────

struct MonitoringControllerInner {
    status: MonitoringStatus,
    last_error: Option<String>,
    /// Active capture worker, present only while Running or Stopping.
    worker: Option<MssCaptureWorker>,
    capture_fps: u32,
    /// HSV 检测引擎
    detection_engine: Option<DetectionEngine>,
    /// 最新检测结果
    latest_detection: Option<DetectionResult>,
}

impl MonitoringControllerInner {
    /// Lock ordering: `inner` must always be acquired BEFORE `latest_frame`.
    /// Never hold `latest_frame` while acquiring `inner`.
    fn to_snapshot(&self) -> MonitoringSnapshot {
        let last_frame_at_ms = self
            .worker
            .as_ref()
            .and_then(|w| w.get_latest_frame())
            .map(|f| f.captured_at_ms);

        MonitoringSnapshot {
            status: self.status.clone(),
            last_error: self.last_error.clone(),
            capture_fps: self.capture_fps,
            last_frame_at_ms,
        }
    }
}

// ─── Public managed state ────────────────────────────────────────────────────

/// Tauri managed state for the monitoring lifecycle.
/// Register with `.manage(MonitoringController::default())` in `lib.rs`.
pub struct MonitoringController {
    inner: Mutex<MonitoringControllerInner>,
}

impl Default for MonitoringController {
    fn default() -> Self {
        Self {
            inner: Mutex::new(MonitoringControllerInner {
                status: MonitoringStatus::Idle,
                last_error: None,
                worker: None,
                capture_fps: 5,
                detection_engine: None,
                latest_detection: None,
            }),
        }
    }
}

// ─── Pure helper functions (unit-testable without hardware) ──────────────────

/// Return the first ROI configured for MSS capture, or `None`.
fn find_mss_roi(config: &MonitorConfig) -> Option<&RoiConfig> {
    config.rois.iter().find(|r| r.capture_mode == CaptureMode::MSS)
}

/// Return `true` when monitoring is already starting or running.
fn is_already_running(status: &MonitoringStatus) -> bool {
    matches!(
        status,
        MonitoringStatus::Starting | MonitoringStatus::Running
    )
}

// ─── Tauri commands ──────────────────────────────────────────────────────────

/// Start MSS monitoring from the given config.
///
/// Steps:
/// 1. Freeze config.
/// 2. Locate the first MSS ROI — error if none.
/// 3. Atomically check for duplicate start — error if already Starting/Running.
/// 4. Transition to `Starting` and emit the event.
/// 5. Create and start the worker outside the lock.
/// 6. Transition to `Running` (or `Error`) and emit the final event.
#[tauri::command]
pub async fn start_monitoring(
    config: MonitorConfig,
    state: tauri::State<'_, MonitoringController>,
    app: tauri::AppHandle,
) -> Result<MonitoringSnapshot, String> {
    // Step 1 – runtime freeze (deep copy).
    let frozen = create_frozen_config(&config);

    // Step 2 – require at least one MSS ROI.
    let mss_roi = match find_mss_roi(&frozen) {
        Some(roi) => roi.clone(),
        None => {
            let error = "未配置 MSS 监控区域".to_string();
            let _ = app.emit(crate::events::MONITORING_ERROR, &error);
            return Err(error);
        }
    };

    let capture_fps = frozen.capture_fps;

    // 验证所有颜色规则配置
    for rule in &mss_roi.color_rules {
        validate_color_match_config(rule)?;
    }

    // Step 3 & 4 – atomic check-then-transition under the lock.
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;

        if is_already_running(&inner.status) {
            let error = "监控已经在运行".to_string();
            let _ = app.emit(crate::events::MONITORING_ERROR, &error);
            return Err(error);
        }

        inner.status = MonitoringStatus::Starting;
        inner.last_error = None;
        inner.capture_fps = capture_fps;
    }

    // Emit Starting before any blocking work.
    let _ = app.emit(
        crate::events::MONITORING_STATUS,
        &MonitoringSnapshot {
            status: MonitoringStatus::Starting,
            last_error: None,
            capture_fps,
            last_frame_at_ms: None,
        },
    );

    // Step 5 – create and start worker outside the state lock.
    let mut worker =
        MssCaptureWorker::new(mss_roi.id.clone(), mss_roi.region.clone(), capture_fps);

    match worker.start() {
        Ok(()) => {
            // Step 6a – transition to Running.
            let snapshot = {
                let mut inner = state
                    .inner
                    .lock()
                    .map_err(|_| "内部状态锁定失败".to_string())?;
                inner.status = MonitoringStatus::Running;
                inner.worker = Some(worker);
                inner.detection_engine = Some(DetectionEngine::new(mss_roi.color_rules.clone()));
                inner.latest_detection = None;
                inner.to_snapshot()
            };
            let _ = app.emit(crate::events::MONITORING_STATUS, &snapshot);
            Ok(snapshot)
        }
        Err(e) => {
            // Step 6b – transition to Error.
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
    }
}

/// Stop monitoring and join the capture thread before reporting stopped.
///
/// Steps:
/// 1. Take the worker out of state and transition to `Stopping`.
/// 2. Emit `Stopping`.
/// 3. Call `worker.stop()` — this **blocks** until the capture thread exits.
/// 4. Transition to `Stopped` (or `Error`) and emit the final event.
#[tauri::command]
pub async fn stop_monitoring(
    state: tauri::State<'_, MonitoringController>,
    app: tauri::AppHandle,
) -> Result<MonitoringSnapshot, String> {
    // Pre-check – reject if monitoring is not in an active state.
    {
        let inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;
        if !matches!(
            inner.status,
            MonitoringStatus::Running | MonitoringStatus::Starting
        ) {
            let error = "监控未在运行中".to_string();
            let _ = app.emit(crate::events::MONITORING_ERROR, &error);
            return Err(error);
        }
    }

    // Step 1 – take the worker and mark Stopping atomically.
    let worker_opt = {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;
        inner.status = MonitoringStatus::Stopping;
        inner.worker.take()
    };

    // Step 2 – emit Stopping.
    {
        let inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;
        let _ = app.emit(crate::events::MONITORING_STATUS, &inner.to_snapshot());
    }

    // Step 3 – join the worker thread (outside the state lock to avoid blocking other callers).
    if let Some(mut worker) = worker_opt {
        if let Err(e) = worker.stop() {
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
            return Err(e);
        }
    }

    // Step 4 – transition to Stopped.
    let snapshot = {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;
        inner.status = MonitoringStatus::Stopped;
        inner.detection_engine = None;
        inner.latest_detection = None;
        inner.to_snapshot()
    };
    let _ = app.emit(crate::events::MONITORING_STATUS, &snapshot);
    Ok(snapshot)
}

/// Return the current monitoring snapshot without side effects.
#[tauri::command]
pub async fn get_monitoring_status(
    state: tauri::State<'_, MonitoringController>,
) -> Result<MonitoringSnapshot, String> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| "内部状态锁定失败".to_string())?;
    Ok(inner.to_snapshot())
}

/// 评估最新捕获帧的检测结果
///
/// 获取最新捕获帧，使用检测引擎评估，发射 detection-result 事件，
/// 并返回检测结果。采用 clone-out-of-lock 模式避免长时间持有 Mutex。
#[tauri::command]
pub async fn evaluate_latest_frame(
    state: tauri::State<'_, MonitoringController>,
    app: tauri::AppHandle,
) -> Result<Option<DetectionResult>, String> {
    // Clone engine and get frame outside of long-held lock
    let (engine, frame_opt) = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;

        let engine = inner
            .detection_engine
            .as_ref()
            .ok_or_else(|| "检测引擎未初始化".to_string())?
            .clone();

        let frame = inner.worker.as_ref().and_then(|w| w.get_latest_frame());

        (engine, frame)
    };

    let Some(frame) = frame_opt else {
        return Ok(None);
    };

    let result = engine.evaluate_frame(&frame);

    // 存储最新检测结果（brief lock to update state）
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "内部状态锁定失败".to_string())?;
        inner.latest_detection = Some(result.clone());
    }

    // 发射检测结果事件（per D-11）
    let _ = app.emit(crate::events::DETECTION_RESULT, &result);

    Ok(Some(result))
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColorMatchConfig, MonitorConfig, Rect, RoiConfig};

    // ── Test helpers ─────────────────────────────────────────────────────────

    fn config_with_mss_roi() -> MonitorConfig {
        let mut cfg = MonitorConfig::default();
        cfg.rois.push(RoiConfig {
            id: "roi-mss-1".to_string(),
            capture_mode: CaptureMode::MSS,
            region: Rect {
                x: 10,
                y: 20,
                width: 200,
                height: 100,
            },
            ..RoiConfig::default()
        });
        cfg
    }

    fn config_with_wgc_roi_only() -> MonitorConfig {
        let mut cfg = MonitorConfig::default();
        cfg.rois.push(RoiConfig {
            id: "roi-wgc-1".to_string(),
            capture_mode: CaptureMode::WGC,
            region: Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 80,
            },
            ..RoiConfig::default()
        });
        cfg
    }

    // ── MonitoringController default state ───────────────────────────────────

    #[test]
    fn controller_defaults_to_idle() {
        let controller = MonitoringController::default();
        let inner = controller.inner.lock().unwrap();
        assert_eq!(inner.status, MonitoringStatus::Idle);
        assert!(inner.last_error.is_none());
        assert!(inner.worker.is_none());
    }

    #[test]
    fn controller_default_snapshot_reflects_idle() {
        let controller = MonitoringController::default();
        let inner = controller.inner.lock().unwrap();
        let snap = inner.to_snapshot();
        assert_eq!(snap.status, MonitoringStatus::Idle);
        assert!(snap.last_error.is_none());
        assert!(snap.last_frame_at_ms.is_none());
        assert_eq!(snap.capture_fps, 5);
    }

    // ── find_mss_roi ─────────────────────────────────────────────────────────

    #[test]
    fn find_mss_roi_returns_first_mss_entry() {
        let cfg = config_with_mss_roi();
        let roi = find_mss_roi(&cfg);
        assert!(roi.is_some());
        assert_eq!(roi.unwrap().id, "roi-mss-1");
    }

    #[test]
    fn find_mss_roi_returns_none_when_only_wgc() {
        let cfg = config_with_wgc_roi_only();
        assert!(find_mss_roi(&cfg).is_none());
    }

    #[test]
    fn find_mss_roi_returns_none_for_empty_rois() {
        let cfg = MonitorConfig::default();
        assert!(find_mss_roi(&cfg).is_none());
    }

    #[test]
    fn find_mss_roi_picks_first_of_multiple() {
        let mut cfg = MonitorConfig::default();
        cfg.rois.push(RoiConfig {
            id: "first".to_string(),
            capture_mode: CaptureMode::MSS,
            ..RoiConfig::default()
        });
        cfg.rois.push(RoiConfig {
            id: "second".to_string(),
            capture_mode: CaptureMode::MSS,
            ..RoiConfig::default()
        });
        assert_eq!(find_mss_roi(&cfg).unwrap().id, "first");
    }

    // ── is_already_running ───────────────────────────────────────────────────

    #[test]
    fn is_already_running_true_for_starting() {
        assert!(is_already_running(&MonitoringStatus::Starting));
    }

    #[test]
    fn is_already_running_true_for_running() {
        assert!(is_already_running(&MonitoringStatus::Running));
    }

    #[test]
    fn is_already_running_false_for_idle() {
        assert!(!is_already_running(&MonitoringStatus::Idle));
    }

    #[test]
    fn is_already_running_false_for_stopped() {
        assert!(!is_already_running(&MonitoringStatus::Stopped));
    }

    #[test]
    fn is_already_running_false_for_stopping() {
        assert!(!is_already_running(&MonitoringStatus::Stopping));
    }

    #[test]
    fn is_already_running_false_for_error() {
        assert!(!is_already_running(&MonitoringStatus::Error));
    }

    // ── Error message content ────────────────────────────────────────────────

    #[test]
    fn no_mss_roi_error_contains_required_text() {
        let cfg = config_with_wgc_roi_only();
        let frozen = create_frozen_config(&cfg);
        let result: Result<(), String> = find_mss_roi(&frozen)
            .map(|_| ())
            .ok_or_else(|| "未配置 MSS 监控区域".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未配置 MSS 监控区域"));
    }

    #[test]
    fn duplicate_start_error_contains_required_text() {
        // Verify the error string literal used in start_monitoring is correct.
        let err = "监控已经在运行".to_string();
        assert!(err.contains("监控已经在运行"));
    }

    // ── create_frozen_config independence ────────────────────────────────────

    #[test]
    fn frozen_config_is_independent_of_original() {
        let mut cfg = MonitorConfig::default();
        cfg.capture_fps = 10;
        let frozen = create_frozen_config(&cfg);
        assert_eq!(frozen.capture_fps, 10);
        // Mutating original must not affect the frozen copy.
        cfg.capture_fps = 20;
        assert_eq!(frozen.capture_fps, 10);
    }

    // ── Status transition correctness ────────────────────────────────────────

    #[test]
    fn all_six_status_variants_are_distinct() {
        let statuses = [
            MonitoringStatus::Idle,
            MonitoringStatus::Starting,
            MonitoringStatus::Running,
            MonitoringStatus::Stopping,
            MonitoringStatus::Stopped,
            MonitoringStatus::Error,
        ];
        // Each variant must not equal any other.
        for (i, a) in statuses.iter().enumerate() {
            for (j, b) in statuses.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn controller_state_can_transition_idle_to_starting() {
        let controller = MonitoringController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            assert!(!is_already_running(&inner.status));
            inner.status = MonitoringStatus::Starting;
        }
        let inner = controller.inner.lock().unwrap();
        assert!(is_already_running(&inner.status));
    }

    #[test]
    fn controller_state_can_transition_to_stopped() {
        let controller = MonitoringController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.status = MonitoringStatus::Stopping;
        }
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.status = MonitoringStatus::Stopped;
        }
        let inner = controller.inner.lock().unwrap();
        assert_eq!(inner.status, MonitoringStatus::Stopped);
        assert!(!is_already_running(&inner.status));
    }

    #[test]
    fn controller_records_error_message() {
        let controller = MonitoringController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.status = MonitoringStatus::Error;
            inner.last_error = Some("捕获失败".to_string());
        }
        let inner = controller.inner.lock().unwrap();
        let snap = inner.to_snapshot();
        assert_eq!(snap.status, MonitoringStatus::Error);
        assert_eq!(snap.last_error.as_deref(), Some("捕获失败"));
    }

    // ── Detection integration ─────────────────────────────────────────────

    #[test]
    fn color_rule_validation_rejected_on_start() {
        // Create config with invalid color rule (min_pixels = 0)
        let mut cfg = MonitorConfig::default();
        cfg.rois.push(RoiConfig {
            id: "roi-test".to_string(),
            capture_mode: CaptureMode::MSS,
            region: Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 80,
            },
            color_rules: vec![ColorMatchConfig {
                name: "invalid".to_string(),
                hsv_lower: [0, 120, 120],
                hsv_upper: [15, 255, 255],
                min_pixels: 0, // invalid
                min_ratio: 0.02,
            }],
            ..RoiConfig::default()
        });
        let frozen = create_frozen_config(&cfg);
        let roi = find_mss_roi(&frozen).unwrap();
        // Validation should fail
        for rule in &roi.color_rules {
            let result = validate_color_match_config(rule);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("最小像素数必须大于 0"));
        }
    }

    #[test]
    fn detection_result_event_name_is_correct() {
        assert_eq!(crate::events::DETECTION_RESULT, "detection-result");
    }

    #[test]
    fn controller_inner_carries_detection_engine_field() {
        let controller = MonitoringController::default();
        let inner = controller.inner.lock().unwrap();
        assert!(inner.detection_engine.is_none());
        assert!(inner.latest_detection.is_none());
    }

    #[test]
    fn controller_inner_can_hold_detection_engine() {
        let controller = MonitoringController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.detection_engine = Some(DetectionEngine::new(vec![
                ColorMatchConfig::default_hostile_marker(),
            ]));
        }
        let inner = controller.inner.lock().unwrap();
        assert!(inner.detection_engine.is_some());
    }
}
