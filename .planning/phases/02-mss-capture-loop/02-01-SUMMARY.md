---
phase: 02-mss-capture-loop
plan: 01
subsystem: capture
tags: [rust, xcap, mss, thread, latest-frame-wins, region-capture]

# Dependency graph
requires:
  - phase: 01-foundation-and-config-spine
    provides: [MonitorConfig, Rect, RoiConfig, serde-defaults]
provides:
  - [CapturedFrame struct with ROI metadata and RGBA buffer]
  - [MonitoringStatus enum for lifecycle states]
  - [MssCaptureWorker with std::thread-based capture loop]
  - [Validation helpers for FPS and ROI dimensions]
affects: [02-02-lifecycle-wiring, 03-hsv-detection]

# Tech tracking
tech-stack:
  added: [xcap 0.9 for Windows screen capture]
  patterns: [latest-frame-wins with Arc<Mutex<Option<T>>>, std::thread lifecycle with AtomicBool cancellation]

key-files:
  created: [src-tauri/src/capture/mod.rs, src-tauri/src/capture/mss.rs]
  modified: [src-tauri/Cargo.toml, src-tauri/src/models/config.rs, src-tauri/src/lib.rs]

key-decisions:
  - "XCap Monitor::capture_region for ROI capture instead of full-monitor crop"
  - "Latest-frame-wins storage to prevent unbounded frame queue"
  - "AtomicBool cancellation + JoinHandle for clean thread shutdown"

patterns-established:
  - "Pattern 1: Worker lifecycle - start spawns thread, stop sets flag and joins"
  - "Pattern 2: Latest-frame-wins - Arc<Mutex<Option<T>>> for single-slot shared state"
  - "Pattern 3: Chinese error messages for user-facing validation failures"

requirements-completed: [CAP-01]

# Metrics
duration: 21min
completed: 2026-04-27
---

# Phase 2 Plan 1: MSS Capture Service and Frame Contract Summary

**XCap-based MSS desktop region capture with thread-based worker, latest-frame-wins delivery, and 5 FPS default cadence**

## Performance

- **Duration:** 21 min
- **Started:** 2026-04-27T08:56:41Z
- **Completed:** 2026-04-27T09:17:59Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added XCap dependency and configurable capture_fps (default 5) to MonitorConfig
- Implemented CapturedFrame, MonitoringStatus, and MssCaptureWorker with clean lifecycle
- Created validation helpers for FPS (1-30) and ROI dimensions with Chinese error messages
- Implemented latest-frame-wins storage to prevent memory growth
- Added unit tests covering all validation logic (6 tests, all passing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add XCap dependency and capture_fps config** - `2ea4b3c` (feat)
2. **Task 2: Create capture module and frame contract** - `0e40bf9` (feat)
3. **Task 3: Add capture service unit tests** - (included in Task 2 commit)

**Plan metadata:** (to be committed after summary creation)

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added xcap = "0.9" dependency
- `src-tauri/src/models/config.rs` - Added capture_fps: u32 field with default 5
- `src-tauri/src/capture/mod.rs` - Module exports for capture types
- `src-tauri/src/capture/mss.rs` - Core MSS capture service with 320 lines:
  - CapturedFrame struct with ROI metadata and RGBA buffer
  - MonitoringStatus enum (Idle, Starting, Running, Stopping, Stopped, Error)
  - MssCaptureWorker with thread lifecycle and latest-frame-wins
  - Validation helpers for FPS and ROI dimensions
  - 6 unit tests covering validation logic
- `src-tauri/src/lib.rs` - Added capture module declaration

## Decisions Made

- Used XCap Monitor::capture_region for ROI capture to avoid full-monitor crop overhead
- Implemented latest-frame-wins with Arc<Mutex<Option<CapturedFrame>>> to prevent unbounded frame queue
- Used Arc<AtomicBool> for cancellation signal and JoinHandle for clean thread shutdown
- Validated ROI coordinates are non-negative before passing to XCap (u32 vs i32 conversion)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- XCap's capture_region expects u32 parameters but Rect uses i32 for x/y - fixed by adding non-negative validation and casting
- XCap's ImageBuffer type doesn't have to_rgba() method - fixed by using as_raw() to extract RGBA bytes directly

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend capture service and frame contract ready for Phase 3 (HSV Detection)
- No Tauri commands registered yet - will be added in Plan 02-02 (lifecycle wiring)
- No frontend UI integration yet - will be added in Plan 02-02

---
*Phase: 02-mss-capture-loop*
*Plan: 01*
*Completed: 2026-04-27*
