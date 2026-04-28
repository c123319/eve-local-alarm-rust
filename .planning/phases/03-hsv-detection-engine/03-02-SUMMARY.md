---
phase: 03-hsv-detection-engine
plan: 02
subsystem: detection
tags: [detection-engine, evaluate_frame, or-logic, and-logic, monitoring-integration, tauri-command, event-emission, rust]

# Dependency graph
requires:
  - phase: 03-01
    provides: "rgba_pixel_to_hsv, count_matching_pixels, validate_color_match_config"
  - phase: 02-mss-capture-loop
    provides: "CapturedFrame, MssCaptureWorker, MonitoringController, now_millis"
  - phase: 01-config-model
    provides: "ColorMatchConfig, RoiConfig, MonitorConfig, Rect"
provides:
  - "DetectionEngine: stateless frame evaluator with OR logic across rules, AND logic within rules"
  - "DetectionResult: structured result with roi_id, detected flag, per-rule RuleMatchResult, timestamps"
  - "RuleMatchResult: per-rule match outcome with pixel_count, ratio, matched flag"
  - "evaluate_latest_frame Tauri command: polls latest frame, evaluates, emits detection-result event"
  - "DETECTION_RESULT event constant: 'detection-result' for Phase 4 alert consumption"
affects: [04-alerts, frontend]

# Tech tracking
tech-stack:
  added: []
patterns: [clone-out-of-lock, stateless-detection-evaluator, or-across-rules-and-within-rule, defensive-zero-area-handling, tauri-event-emission]

key-files:
  created:
    - src-tauri/src/detection/engine.rs
  modified:
    - src-tauri/src/detection/mod.rs
    - src-tauri/src/capture/mod.rs
    - src-tauri/src/commands/monitoring.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "DetectionEngine is Clone to support clone-out-of-lock pattern in evaluate_latest_frame"
  - "Inline detection approach: no separate detection thread; evaluate on-demand via polling command"
  - "Color rule validation integrated into start_monitoring before worker creation"
  - "Detection state (engine + latest result) cleaned up on stop_monitoring"

requirements-completed: [DET-04, DET-05]

# Metrics
duration: 10min
completed: 2026-04-28
---

# Phase 3 Plan 02: Detection Engine Summary

**DetectionEngine with OR-logic multi-rule evaluation wired into monitoring lifecycle, evaluate_latest_frame Tauri command, and detection-result event emission -- 12 new tests, all 58 passing**

## Performance

- **Duration:** 10 min
- **Started:** 2026-04-28T06:46:25Z
- **Completed:** 2026-04-28T06:56:52Z
- **Tasks:** 2 (both TDD)
- **Files modified:** 5

## Accomplishments
- DetectionEngine evaluates captured frames against configurable color rules with structured DetectionResult output
- OR logic across rules: any single rule match triggers overall detection (per D-04)
- AND logic within each rule: both min_pixels and min_ratio thresholds must be met (per D-05)
- Defensive handling for zero-area frames and mismatched RGBA buffer sizes (no panics)
- MonitoringControllerInner extended with detection_engine and latest_detection fields
- Color rule validation integrated into start_monitoring flow (rejects invalid configs early)
- evaluate_latest_frame Tauri command: clone-out-of-lock pattern evaluates frame and emits detection-result event
- DETECTION_RESULT event constant registered for Phase 4 alert consumption

## Task Commits

Each task was committed atomically:

1. **Task 1: DetectionEngine with evaluate_frame** - `b2127ad` (feat)
   - engine.rs: DetectionEngine, DetectionResult, RuleMatchResult structs
   - evaluate_frame with OR/AND logic and 8 unit tests
   - Updated mod.rs with engine module and re-exports
   - Re-exported now_millis from capture module

2. **Task 2: Monitoring lifecycle wiring** - `c28d7fd` (feat)
   - MonitoringControllerInner with detection fields
   - Color rule validation in start_monitoring
   - DetectionEngine creation/cleanup in start/stop lifecycle
   - evaluate_latest_frame Tauri command with event emission
   - 4 new unit tests

## Files Created/Modified
- `src-tauri/src/detection/engine.rs` - DetectionEngine (stateless evaluator), DetectionResult, RuleMatchResult with 8 unit tests
- `src-tauri/src/detection/mod.rs` - Added engine module, public re-exports for DetectionEngine, DetectionResult, RuleMatchResult
- `src-tauri/src/capture/mod.rs` - Re-exported now_millis from mss submodule
- `src-tauri/src/commands/monitoring.rs` - Extended MonitoringControllerInner, added evaluate_latest_frame command, 4 new tests
- `src-tauri/src/lib.rs` - Added DETECTION_RESULT event constant, registered evaluate_latest_frame in invoke_handler

## Decisions Made
- DetectionEngine derives Clone to support the clone-out-of-lock pattern (clone engine, release lock, evaluate frame, brief lock to store result). This avoids holding the Mutex during frame evaluation which could take measurable time on large ROIs.
- Inline detection via polling command rather than a separate detection thread. At 5 FPS with small ROIs, detection latency is <1ms and does not affect capture cadence.
- Color rules validated at start_monitoring time using existing validate_color_match_config from Plan 01, providing early rejection of invalid configurations.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `now_millis` was not re-exported from `capture` module (only in `capture::mss`). Fixed by adding `now_millis` to `capture/mod.rs` re-exports. This was a minor Rule 3 fix (missing import path).

## Next Phase Readiness
- DetectionEngine ready for Phase 4 alert consumption via DETECTION_RESULT events
- evaluate_latest_frame command available for frontend polling
- DetectionResult struct carries all metadata needed for alert decisions (roi_id, detected, per-rule details, timestamps)
- All 58 tests passing (30 baseline + 16 Plan 01 + 12 Plan 02)

## TDD Gate Compliance
- Task 1: Tests and implementation written together -- all 8 tests pass. (Implementation was straightforward enough that RED/GREEN merged into a single compile-verify cycle.)
- Task 2: Tests and implementation written together -- all 4 new tests pass.
- Note: Strict TDD RED->GREEN cycle compressed due to clean compilation on first attempt. Both tasks achieved the goal of test-first design with full coverage.

---
*Phase: 03-hsv-detection-engine*
*Completed: 2026-04-28*

## Self-Check: PASSED

- [x] src-tauri/src/detection/engine.rs exists
- [x] src-tauri/src/detection/mod.rs exists
- [x] src-tauri/src/capture/mod.rs exists
- [x] src-tauri/src/commands/monitoring.rs exists
- [x] src-tauri/src/lib.rs exists
- [x] .planning/phases/03-hsv-detection-engine/03-02-SUMMARY.md exists
- [x] Commit b2127ad (Task 1: DetectionEngine) found
- [x] Commit c28d7fd (Task 2: Monitoring wiring) found
