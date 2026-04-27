# Phase 02: MSS Capture Loop - Research

**Researched:** 2026-04-27
**Status:** Ready for planning

## Research Question

What must be known to plan Phase 2 well: a working single-ROI MSS-style capture loop using XCap, configurable cadence, `std::thread` lifecycle control, latest-frame-wins frame delivery, and minimal Chinese start/stop UI.

## Key Findings

### XCap capture API

- Use `xcap::Monitor::all()` to enumerate monitors and select the target monitor for MSS capture.
- Use `Monitor::capture_region(x, y, width, height)` for ROI capture. This is preferred over full-monitor capture followed by cropping.
- Region capture should validate that ROI dimensions are non-zero and fit the selected monitor bounds before entering the loop.
- Captured image data should be converted into an internal Rust frame type rather than sent directly to the frontend in Phase 2; Phase 3 detection will consume the frame contract.

Primary references:
- `https://github.com/nashaofu/xcap`
- `https://github.com/nashaofu/xcap/blob/master/README.md#region-capture`
- `https://github.com/nashaofu/xcap/issues/102`
- `https://github.com/nashaofu/xcap/compare/v0.5.2...v0.6.0`

### Tauri lifecycle pattern

- Keep start/stop as Tauri commands registered through the existing `tauri::generate_handler!` pattern.
- Store monitoring state in managed Tauri app state so repeated commands can detect already-running/stopped states.
- Use a cancellation signal (`Arc<AtomicBool>` or equivalent) plus a `JoinHandle` for clean `std::thread` shutdown.
- Emit status/error events to the frontend with stable event names; the frontend should listen and display backend truth, not local-only state.

Primary references:
- `https://v2.tauri.app/develop/calling-rust/`
- `https://v2.tauri.app/develop/calling-frontend/`
- `https://v2.tauri.app/concept/inter-process-communication`
- `https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/async_runtime.rs`
- `https://github.com/tauri-apps/tauri/blob/dev/examples/api/src-tauri/src/menu_plugin.rs`

### Codebase integration

- Existing command pattern lives in `src-tauri/src/commands/config.rs`, `src-tauri/src/commands/dpi.rs`, `src-tauri/src/commands/mod.rs`, and `src-tauri/src/lib.rs`.
- Existing config model already contains `CaptureMode::MSS`, `RoiConfig`, and physical-pixel `Rect`.
- Add a global capture cadence field to `MonitorConfig` for Phase 2, e.g. `capture_fps: u32`, defaulting to `5`.
- Add capture service modules under `src-tauri/src/capture/` so Phase 3 can attach detection without bloating command files.
- Extend `SettingsPanel.tsx` with a minimal `监控控制` section following `02-UI-SPEC.md`.

## Recommended Implementation Shape

### Rust modules

- `src-tauri/src/capture/mod.rs`
  - Re-export capture types and MSS capture worker.
- `src-tauri/src/capture/mss.rs`
  - Define `CapturedFrame`, `CaptureStatus`, `MonitoringState`, and `MssCaptureWorker` or equivalent.
  - Own thread lifecycle and latest-frame handoff.
- `src-tauri/src/commands/monitoring.rs`
  - Expose `start_monitoring`, `stop_monitoring`, and `get_monitoring_status` commands.
- `src-tauri/src/models/config.rs`
  - Add configurable capture FPS with serde default safety.
- `src-tauri/src/lib.rs`
  - Manage monitoring state and register commands/events.

### Frontend modules

- `src/components/SettingsPanel.tsx`
  - Add monitoring state, event listeners, start/stop invoke handlers, visible-region warning, FPS display, and error display.

## Validation Architecture

### Observable truths

1. `CAP-01`: A Rust capture service can capture a visible physical-pixel ROI in MSS mode using XCap.
2. `UI-02`: The main app UI exposes start and stop monitoring controls.
3. Stop is clean: the monitoring thread receives a stop signal and no orphaned worker remains.
4. Captured frames are available through a stable Rust-side contract containing ROI id, timestamp, dimensions, and buffer data.

### Required validation commands

- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml capture`
- `cargo test --manifest-path src-tauri/Cargo.toml monitoring`
- `npm run build`

### Required static checks

- `src-tauri/Cargo.toml` contains `xcap`.
- `src-tauri/src/lib.rs` registers `start_monitoring`, `stop_monitoring`, and `get_monitoring_status`.
- `src/components/SettingsPanel.tsx` contains Chinese copy `监控控制`, `开始监控`, `停止监控`, and `MSS 模式仅捕获屏幕可见区域`.

## Risks and Mitigations

- **DPI/coordinate mismatch:** keep stored ROI coordinates as physical pixels and validate against monitor bounds before capture.
- **Frame backlog:** use latest-frame-wins bounded handoff, not an unbounded queue.
- **Thread leaks:** stop through a cancellation flag and join the worker before reporting stopped.
- **Tauri bridge overload:** do not stream frame images to frontend in Phase 2; emit only lifecycle/status/error events.

## RESEARCH COMPLETE
