---
phase: 02-mss-capture-loop
fixed_at: 2026-04-28T12:30:00Z
review_path: .planning/phases/02-mss-capture-loop/02-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-04-28T12:30:00Z
**Source review:** .planning/phases/02-mss-capture-loop/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Frontend-backend field name mismatch for ROI region

**Files modified:** `src/components/SettingsPanel.tsx`
**Commit:** 786e3c8
**Applied fix:** Renamed `rect` to `region` in the TypeScript `RoiConfig` interface (line 31) to match the Rust backend `RoiConfig.region` field. This was a showstopper -- serde deserialization silently defaulted the region to all zeros, causing ROI validation to always fail.

### WR-01: Mutex unwrap() calls can panic and crash the application

**Files modified:** `src-tauri/src/capture/mss.rs`
**Commit:** 1f05ae4
**Applied fix:** Replaced three `.unwrap()` calls with poison-safe alternatives:
- `get_latest_frame()`: uses `match` with `poisoned.into_inner()` recovery
- Capture loop `latest_frame.lock()`: uses `match` with `poisoned.into_inner()` recovery
- `now_millis()`: uses `.unwrap_or_default()` instead of `.unwrap()`

### WR-02: stop_monitoring allows invalid state transitions

**Files modified:** `src-tauri/src/commands/monitoring.rs`
**Commit:** cab7ea3
**Applied fix:** Added a pre-check before the `Stopping` transition that verifies the current status is `Running` or `Starting`. If not, emits a `monitoring-error` event and returns an error ("监控未在运行中"), preventing nonsensical state transitions from Idle/Stopped/Error states.

### WR-03: get_latest_frame() called while holding inner lock in to_snapshot()

**Files modified:** `src-tauri/src/commands/monitoring.rs`
**Commit:** aeec15b
**Applied fix:** Added a doc comment on `MonitoringControllerInner::to_snapshot()` documenting the lock ordering contract: `inner` must always be acquired before `latest_frame`, and `latest_frame` must never be held while acquiring `inner`.

---

_Fixed: 2026-04-28T12:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
