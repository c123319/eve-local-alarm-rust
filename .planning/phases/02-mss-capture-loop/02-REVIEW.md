---
phase: 02-mss-capture-loop
reviewed: 2026-04-28T12:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - src-tauri/Cargo.toml
  - src-tauri/src/capture/mod.rs
  - src-tauri/src/capture/mss.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/monitoring.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/models/config.rs
  - src/components/SettingsPanel.tsx
findings:
  critical: 1
  warning: 3
  info: 3
  total: 7
status: issues_found
---

# Phase 2: Code Review Report

**Reviewed:** 2026-04-28T12:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Reviewed the Phase 2 MSS capture loop implementation across 8 files (Rust backend + React/TypeScript frontend). The architecture is sound: a mutex-guarded monitoring controller manages a worker thread with atomic cancellation, and the state machine transitions (Idle -> Starting -> Running -> Stopping -> Stopped) are well-designed with good event emission.

However, one **critical** field-name mismatch between frontend and backend will prevent monitoring from starting at all. Three additional warnings relate to potential panic paths and misleading state transitions. The codebase demonstrates strong testing discipline with comprehensive unit tests for both the capture validation and monitoring controller logic.

## Critical Issues

### CR-01: Frontend-backend field name mismatch for ROI region

**File:** `src/components/SettingsPanel.tsx:31` and `src-tauri/src/models/config.rs:42`
**Issue:** The TypeScript `RoiConfig` interface defines the region field as `rect` (line 31), but the Rust `RoiConfig` struct uses `region` (config.rs line 42). When the frontend sends config to `start_monitoring`, Tauri's serde deserialization will look for a field named `region` but the JSON payload contains `rect`. Serde's `#[serde(default)]` on `RoiConfig` means `region` silently defaults to `Rect::default()` (all zeros: x=0, y=0, width=0, height=0). The zero-dimension ROI then fails `validate_roi`, producing the error "ROI width and height must be greater than 0" -- which misleads the user into thinking their ROI config is wrong when the real problem is a field name mismatch.

This is a showstopper: monitoring can never start from the UI because the ROI region is always lost during serialization.

**Fix:**

Option A -- Fix the frontend to match the backend (recommended):
```typescript
// src/components/SettingsPanel.tsx, line 28-32
interface RoiConfig {
  id: string;
  name: string;
  capture_mode: string;
  region: { x: number; y: number; width: number; height: number };
}
```

Option B -- Add `#[serde(rename = "rect")]` to the Rust field:
```rust
// src-tauri/src/models/config.rs, line 42
#[serde(rename = "rect")]
pub region: Rect,
```

Option A is preferred because `region` is the name used everywhere else in the Rust codebase (validation, capture, etc.).

## Warnings

### WR-01: Mutex unwrap() calls can panic and crash the application

**File:** `src-tauri/src/capture/mss.rs:131`, `src-tauri/src/capture/mss.rs:156`, `src-tauri/src/capture/mss.rs:242`
**Issue:** Three `.lock().unwrap()` calls on `Mutex` guards. If the capture thread panics while holding the lock (e.g., due to a bug in `capture_frame` or an unexpected xcap error that propagates as a panic), the mutex becomes poisoned. Any subsequent `.unwrap()` call on that mutex will panic, crashing the entire Tauri application. This is especially dangerous for lines 131 and 156 which run in the hot capture loop.

Line 242 (`now_millis()`) uses `.unwrap()` on `SystemTime::duration_since(UNIX_EPOCH)`, which can theoretically fail if the system clock is set before epoch.

**Fix:**
```rust
// For Mutex locks, use a custom handler that logs and returns a sentinel:
let mut guard = match self.latest_frame.lock() {
    Ok(g) => g,
    Err(poisoned) => poisoned.into_inner(), // recover the last value
};
```

For `now_millis()`:
```rust
pub fn now_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
```

### WR-02: stop_monitoring allows invalid state transitions

**File:** `src-tauri/src/commands/monitoring.rs:183-235`
**Issue:** `stop_monitoring` unconditionally transitions the state to `Stopping` (line 193) regardless of the current status. If called when monitoring is already `Idle`, `Stopped`, or `Error`, it emits a misleading `Stopping` event and then immediately transitions to `Stopped`. This pollutes the event stream with nonsensical state transitions and could confuse the frontend UI. Additionally, calling stop while already `Stopping` (from a concurrent request) is not guarded.

**Fix:** Add a pre-check before transitioning:
```rust
// Before line 193, add:
let current_status = {
    let inner = state.inner.lock().map_err(|_| "内部状态锁定失败".to_string())?;
    inner.status.clone()
};

if !matches!(current_status, MonitoringStatus::Running | MonitoringStatus::Starting) {
    let error = "监控未在运行中".to_string();
    let _ = app.emit(crate::events::MONITORING_ERROR, &error);
    return Err(error);
}
```

### WR-03: get_latest_frame() called while holding inner lock in to_snapshot()

**File:** `src-tauri/src/commands/monitoring.rs:27-30`
**Issue:** `to_snapshot()` calls `self.worker.as_ref().and_then(|w| w.get_latest_frame())` which acquires the `latest_frame` Mutex inside `MssCaptureWorker`. This call happens while `inner` (the `MonitoringControllerInner` Mutex) is already locked. This creates a lock ordering: `inner` -> `latest_frame`. Meanwhile, the capture loop (mss.rs line 156) acquires `latest_frame` without holding `inner`. The ordering is consistent today, but if any future code path acquires `latest_frame` first and then `inner`, this would deadlock. This is a latent risk worth documenting.

**Fix:** Add a comment documenting the lock ordering contract:
```rust
impl MonitoringControllerInner {
    /// Lock ordering: `inner` must always be acquired BEFORE `latest_frame`.
    /// Never hold `latest_frame` while acquiring `inner`.
    fn to_snapshot(&self) -> MonitoringSnapshot {
```

## Info

### IN-01: Frontend RoiConfig interface is incomplete

**File:** `src/components/SettingsPanel.tsx:27-32`
**Issue:** The TypeScript `RoiConfig` interface is missing several fields that the Rust backend defines: `color_rules`, `debounce_ms`, and `dpi_invalidation_flags`. While serde's defaulting handles missing fields gracefully, this means the frontend cannot configure color matching rules or debounce timing, which are core detection parameters.

**Fix:** Complete the interface to match the backend model, or add a comment marking it intentionally partial for this phase:
```typescript
interface RoiConfig {
  id: string;
  name: string;
  capture_mode: string;
  region: { x: number; y: number; width: number; height: number };
  // TODO: Phase 3+ -- color_rules, debounce_ms, dpi_invalidation_flags
}
```

### IN-02: console.error statements in production code

**File:** `src/components/SettingsPanel.tsx:141`, `src/components/SettingsPanel.tsx:153`, `src/components/SettingsPanel.tsx:162`
**Issue:** Three `console.error` calls for failed Tauri invocations. These are useful during development but will appear in the browser DevTools console in production. Consider a structured logging approach or removing before release.

**Fix:** Low priority. Acceptable for current phase. Consider replacing with a debug-mode logger before production release.

### IN-03: Potential division truncation in frame duration calculation

**File:** `src-tauri/src/capture/mss.rs:142`
**Issue:** `1000 / capture_fps as u64` performs integer division. For non-divisor FPS values (e.g., 7 FPS -> 142ms instead of ~143ms), the actual frame rate will be slightly higher than configured. The error is small (at most ~1ms per frame) and unlikely to matter for this use case.

**Fix:** No action needed for this use case. If precise timing matters in the future, consider using `Duration::from_secs_f64(1.0 / capture_fps as f64)`.

---

_Reviewed: 2026-04-28T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
