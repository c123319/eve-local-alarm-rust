# Phase 2: MSS Capture Loop - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2 delivers a working single-ROI MSS-style desktop capture loop that can be started and stopped safely from the app UI. It proves desktop region acquisition, monitoring lifecycle control, and a stable frame-delivery contract for downstream detection. This phase does not implement HSV detection, alerting, ROI drawing UX, WGC multi-window capture, tray controls, or debug artifact generation.

</domain>

<decisions>
## Implementation Decisions

### MSS capture backend
- **D-01:** Use the Rust `xcap` crate as the MSS-style desktop screenshot backend for Phase 2.
- **D-02:** The earlier direct Windows GDI API idea is superseded; downstream research/planning should not design a manual GDI capture layer unless `xcap` proves unusable during implementation.
- **D-03:** Phase 2 capture uses visible desktop/monitor pixels only; the monitored EVE Local region must remain visible on screen. Background/minimized window capture remains WGC scope in Phase 6.
- **D-04:** Prefer XCap's native monitor-region capture API (`Monitor::capture_region(...)`) for ROI frames instead of capturing a full monitor and cropping afterward.

### Capture cadence and runtime lifecycle
- **D-05:** Capture frame rate must be configurable rather than hardcoded. Planning may choose the exact config field placement, but the UI/runtime must support a user-adjustable capture cadence.
- **D-06:** Default capture cadence should stay conservative for local chat monitoring, targeting low CPU usage over high FPS. A 5-10 FPS default range is acceptable unless research finds a stronger crate-specific recommendation.
- **D-07:** The MSS capture loop should run on `std::thread`, because capture is blocking and this phase should keep blocking work out of the Tauri command path.
- **D-08:** Start/stop must be clean and repeatable: stopping monitoring should signal the capture worker to exit and leave no orphaned capture thread behind.

### Frame delivery contract
- **D-09:** Phase 2 must expose captured frames through a stable Rust-side interface for Phase 3 detection; frontend preview streaming is not required in this phase.
- **D-10:** Use latest-frame-wins semantics with a bounded handoff, so stale frames do not accumulate if downstream consumers are slower than capture.
- **D-11:** The frame contract should preserve enough metadata for downstream detection: ROI id, physical-pixel region, capture timestamp, frame dimensions, and raw/image buffer representation.

### Tauri monitoring controls
- **D-12:** Add explicit Tauri commands for monitoring lifecycle, following the existing command pattern: `start_monitoring`, `stop_monitoring`, and a status query if needed by the UI.
- **D-13:** Start monitoring from a frozen runtime copy of the current config; live edits do not affect an active capture loop until monitoring is stopped and restarted.
- **D-14:** Monitoring status and errors should be pushed to the frontend through Tauri events, while start/stop commands remain request/response calls.

### Phase 2 UI shape
- **D-15:** Phase 2 UI should be minimal and utilitarian: add start/stop controls and monitoring status to the existing Chinese settings/control surface rather than creating a new polished workflow.
- **D-16:** UI copy stays Chinese-first and should clearly explain that MSS mode captures a visible desktop region.
- **D-17:** ROI editing/drawing is out of scope; Phase 2 may use the current/default ROI config values or simple numeric config controls as needed to prove lifecycle and capture.

### DPI and coordinates
- **D-18:** The Phase 1 coordinate contract remains locked: stored/runtime ROI coordinates are physical pixels.
- **D-19:** XCap capture results must be reconciled with the existing physical-pixel `Rect` contract before frame delivery. If monitor-relative vs virtual-desktop coordinates differ, planning must make the conversion explicit.
- **D-20:** DPI invalidation warnings from Phase 1 remain relevant, but this phase only needs to respect the contract; full interactive DPI-correct ROI selection belongs to Phase 5.

### Claude's Discretion
- Exact Rust module names for the capture service, as long as they follow the existing `commands/`, `models/`, and feature-module structure.
- Exact channel implementation (`std::sync::mpsc`, `crossbeam-channel`, or equivalent), as long as it is bounded/latest-frame-wins and works cleanly with `std::thread`.
- Exact monitoring status payload shape, as long as the frontend can show running/stopped/error states clearly.
- Exact default FPS value inside the conservative 5-10 FPS range.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` §Phase 2 — Official Phase 2 goal, success criteria, and two plan slots: MSS capture service/frame contract and frontend-backend lifecycle wiring.
- `.planning/REQUIREMENTS.md` §Capture, §UI and Operations — Requirement IDs `CAP-01` and `UI-02` define the observable user outcomes for this phase.
- `.planning/PROJECT.md` §Target features, §Context, §Constraints — Establishes MSS as visible desktop region capture and names `xcap` or similar as the intended MSS backend.

### Prior locked decisions
- `.planning/phases/01-foundation-and-config-spine/01-CONTEXT.md` §DPI and coordinate contract — Physical pixels are the internal system of record; display-space coordinates convert before storage/runtime use.
- `.planning/phases/01-foundation-and-config-spine/01-CONTEXT.md` §Configuration lifecycle — Monitoring uses a frozen config copy and does not support hot config updates.
- `.planning/phases/01-foundation-and-config-spine/01-VERIFICATION.md` — Confirms config freeze, command wiring, DPI contract, and Chinese settings UI are already available.

### Architecture and research
- `.planning/research/ARCHITECTURE.md` §Capture → Detect → Alert Pipeline — Shows `start_monitoring`, MSS capture on timer, and downstream frame flow.
- `.planning/research/ARCHITECTURE.md` §Threading Model — Allows `std::thread` for blocking capture loops and channel-based coordination.
- `.planning/research/ARCHITECTURE.md` §Suggested Build Order — Places MSS capture before HSV detection and alerts.
- `.planning/research/STACK.md` §Capture — Identifies `xcap` for MSS-style desktop region capture and `windows-capture` for later WGC.
- `.planning/research/PITFALLS.md` §DPI Scaling Issues, §Frame Processing Latency, §Tauri v2 Event Volume — Risks relevant to physical-pixel coordinates, stale-frame dropping, and event throttling.

### External references for planning research
- `xcap` crate/repo: `https://github.com/nashaofu/xcap` — Official Rust screen capture library selected for MSS backend.
- `xcap` monitor example: `https://github.com/nashaofu/xcap/blob/master/examples/monitor.rs` — Shows `xcap::Monitor::all()` and monitor metadata enumeration.
- `xcap` region capture example: `https://github.com/nashaofu/xcap/blob/master/README.md#region-capture` — Shows native `Monitor::capture_region(...)` ROI capture.
- `xcap` monitor source/tests: `https://github.com/nashaofu/xcap/blob/master/src/monitor.rs` — Confirms monitor enumeration and region-related test coverage.
- `xcap` Windows DPI discussion: `https://github.com/nashaofu/xcap/issues/176` — Useful background for Windows per-monitor DPI caveats.
- Tauri runtime/state/events references from research: `https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/async_runtime.rs`, `https://github.com/tauri-apps/tauri/blob/dev/examples/api/src-tauri/src/menu_plugin.rs`, and `https://github.com/tauri-apps/tauri-plugin-log/blob/v1/src/lib.rs` — Useful examples for background runtime, managed state, and Rust-to-frontend events.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/models/config.rs` — `MonitorConfig`, `RoiConfig`, `CaptureMode::MSS`, `Rect`, `ColorMatchConfig`, and `DpiInvalidationFlags` already exist. `RoiConfig` defaults to MSS mode and physical-pixel `Rect` storage.
- `src-tauri/src/dpi/contract.rs` — Provides `PhysicalCoord`, `DisplayCoord`, `DpiInfo`, conversion helpers, and DPI invalidation checks to preserve the Phase 1 coordinate contract.
- `src-tauri/src/commands/config.rs` — Contains `create_frozen_config(config: &MonitorConfig) -> MonitorConfig`, the Phase 2 runtime freeze helper.
- `src/components/SettingsPanel.tsx` — Existing Chinese UI and `invoke` pattern for Tauri commands.

### Established Patterns
- Commands are grouped in `src-tauri/src/commands/*.rs`, re-exported from `commands/mod.rs`, imported in `lib.rs`, and registered with `tauri::generate_handler!`.
- Rust command errors currently use `Result<_, String>` with user-facing Chinese UI messages on the frontend.
- `src-tauri/src/lib.rs` already defines an `events` module and notes future Phase 2+ event channel setup.
- Frontend invokes backend commands with `@tauri-apps/api/core` and stores local UI state in React component state.

### Integration Points
- Add capture service modules under `src-tauri/src/capture/`, with MSS-specific implementation in a submodule such as `capture/mss.rs`.
- Add monitoring lifecycle commands under `src-tauri/src/commands/monitoring.rs` and register them in `src-tauri/src/lib.rs`.
- Add monitoring state management to Tauri app state so repeated start/stop calls can detect already-running/stopped states.
- Extend the existing settings/control UI with a small monitoring section for start, stop, and current status.

</code_context>

<specifics>
## Specific Ideas

- User explicitly changed the backend decision from direct Windows GDI API to **XCap as the MSS screenshot backend**.
- User explicitly selected **configurable frame rate** rather than hardcoded capture cadence.
- User explicitly selected **`std::thread`** for the capture loop.
- The Phase 2 experience should stay practical and minimal: prove monitoring start/stop and frame acquisition before adding detection, alerts, or full ROI selector UX.

</specifics>

<deferred>
## Deferred Ideas

- Direct Windows GDI capture implementation — superseded by XCap decision; revisit only if XCap fails during implementation.
- HSV detection and threshold evaluation — Phase 3.
- Popup/sound/toast alerts and cooldown behavior — Phase 4.
- Interactive ROI drawing, live preview, and overlay UX — Phase 5.
- WGC multi-window/background window capture — Phase 6.

</deferred>

---

*Phase: 02-mss-capture-loop*
*Context gathered: 2026-04-27*
