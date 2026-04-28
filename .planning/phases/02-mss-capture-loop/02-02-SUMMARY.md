---
phase: 02-mss-capture-loop
plan: 02
subsystem: monitoring-lifecycle
tags: [tauri-commands, tauri-events, react, typescript, chinese-ui, mutex-state]

# Dependency graph
requires:
  - phase: 02-mss-capture-loop/02-01
    provides: "MssCaptureWorker, MonitoringSnapshot, MonitoringStatus types, config freeze utility"
provides:
  - "Tauri commands: start_monitoring, stop_monitoring, get_monitoring_status"
  - "MonitoringController managed state with Mutex-guarded inner state"
  - "Event constants: monitoring-status, monitoring-error"
  - "Chinese monitoring controls UI in SettingsPanel"
  - "20 unit tests covering pure helper functions and state transitions"
affects: [03-hsv-detection, alert-system, tray-ui]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Mutex-guarded Tauri managed state", "check-then-transition under lock for command safety", "Tauri event emission on state transitions", "React event listeners with useEffect cleanup"]

key-files:
  created: ["src-tauri/src/commands/monitoring.rs"]
  modified: ["src-tauri/src/commands/mod.rs", "src-tauri/src/lib.rs", "src/components/SettingsPanel.tsx"]

key-decisions:
  - "Worker creation happens outside the state lock to avoid blocking other callers"
  - "stop_monitoring joins the worker thread before returning (blocking but safe)"
  - "Frontend initializes status from get_monitoring_status and updates via event listeners"
  - "Status container uses aria-live=polite for accessibility"

patterns-established:
  - "Command pattern: freeze config -> validate input -> atomic state transition -> do work -> final state transition -> emit event"
  - "Frontend pattern: useEffect for initial fetch + event listeners with cleanup, invoke commands for mutations"

requirements-completed: [CAP-01, UI-02]

# Metrics
duration: 5min
completed: 2026-04-28
---

# Phase 2 Plan 2: Monitoring Lifecycle Commands & UI Summary

**Tauri start/stop/status commands with Mutex-guarded MonitoringController, monitoring-status/error events, and Chinese lifecycle UI with all six states**

## Performance

- **Duration:** 5 min (pre-implemented, verification-only)
- **Started:** 2026-04-28T03:11:18Z
- **Completed:** 2026-04-28T03:11:23Z
- **Tasks:** 3 (all pre-committed, verified in this run)
- **Files modified:** 4

## Accomplishments
- Backend lifecycle commands with frozen config, duplicate-start rejection, and worker join on stop
- 20 unit tests covering find_mss_roi, is_already_running, state transitions, config freeze independence, and error message content
- Chinese monitoring controls UI with all six states (Idle/Starting/Running/Stopping/Stopped/Error), event listeners, and aria-live accessibility

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement monitoring commands and backend state** - `0fc300b` (feat)
2. **Task 2: Add monitoring controls to SettingsPanel** - `ecf1810` (feat)
3. **Task 3: Verify end-to-end lifecycle wiring** - No new commit (verification-only, all wiring confirmed correct)

## Files Created/Modified
- `src-tauri/src/commands/monitoring.rs` - Monitoring lifecycle commands (start/stop/status), MonitoringController state, pure helper functions, 20 unit tests
- `src-tauri/src/commands/mod.rs` - Added monitoring module and re-exports
- `src-tauri/src/lib.rs` - Event constants, MonitoringController managed state, command registration in generate_handler
- `src/components/SettingsPanel.tsx` - Chinese monitoring controls section with event listeners, status display, start/stop buttons, error display

## Decisions Made
- Worker creation outside the state lock avoids blocking other command callers during the potentially slow capture setup
- stop_monitoring blocks until the worker thread exits (joins thread) to guarantee clean shutdown before returning
- Frontend initializes from get_monitoring_status and subscribes to monitoring-status/monitoring-error events for real-time updates
- Starting/Stopping status displayed in darker orange (#e67e22) per UI-SPEC accessibility note

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all code was pre-implemented from prior execution run. This run verified all acceptance criteria pass.

## Verification Results

- `cargo test --manifest-path src-tauri/Cargo.toml monitoring`: 20/20 tests passed
- `cargo check --manifest-path src-tauri/Cargo.toml`: Success (6 pre-existing warnings unrelated to this plan)
- `npm run build`: Success (32 modules transformed, built in 654ms)

## Next Phase Readiness
- Monitoring lifecycle fully wired: frontend controls -> Tauri commands -> MssCaptureWorker
- Ready for Phase 3 HSV detection pipeline (worker currently captures frames but does not detect)
- Event infrastructure (monitoring-status, monitoring-error) ready for detection result emission

## Self-Check: PASSED

- FOUND: .planning/phases/02-mss-capture-loop/02-02-SUMMARY.md
- FOUND: src-tauri/src/commands/monitoring.rs
- FOUND: src/components/SettingsPanel.tsx
- FOUND: commit 0fc300b (feat: monitoring lifecycle commands)
- FOUND: commit ecf1810 (feat: monitoring controls to settings panel)

---
*Phase: 02-mss-capture-loop*
*Completed: 2026-04-28*
