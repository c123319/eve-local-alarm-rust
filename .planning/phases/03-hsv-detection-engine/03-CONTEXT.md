# Phase 3: HSV Detection Engine - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 delivers the HSV color-matching detection engine that converts captured RGBA frames into structured hostile-marker detection results. It implements configurable HSV bounds, pixel-count and ratio thresholds, latest-frame-wins processing, and multi-rule OR evaluation. This phase does not implement alerting (Phase 4), ROI selector UX (Phase 5), WGC multi-window capture (Phase 6), tray controls (Phase 7), debug artifact dumps (Phase 8), or template matching (DETX-01).

</domain>

<decisions>
## Implementation Decisions

### HSV detection approach
- **D-01:** Use pure Rust pixel iteration for HSV color matching. Do not introduce OpenCV in Phase 3. The detection engine iterates the RGBA frame buffer, converts each pixel to HSV, and compares against configurable bounds.
- **D-02:** OpenCV introduction is deferred to Phase 8 (debug/diagnostics) or later for template matching (DETX-01). Phase 3 must not depend on OpenCV build infrastructure.
- **D-03:** HSV conversion uses the standard RGB→HSV formula. Hue range is 0-179 (OpenCV convention half-range, matching the existing `hsv_lower`/`hsv_upper` defaults of [0,120,120] to [15,255,255]).

### Color rule evaluation
- **D-04:** Multiple `ColorMatchConfig` rules within a single ROI use OR logic: if ANY rule matches, the frame counts as a positive detection.
- **D-05:** A rule matches when the count of in-range pixels meets BOTH `min_pixels` AND `min_ratio` thresholds (conjunction within a single rule).

### Detection result model
- **D-06:** Each frame evaluation produces a structured `DetectionResult` containing: whether detection is positive, which rule(s) matched, matched pixel count and ratio per rule, ROI id, and frame timestamp.
- **D-07:** Detection results are delivered through a latest-frame-wins channel, consistent with the capture frame delivery pattern.

### Detection pipeline integration
- **D-08:** Detection runs on the same thread as capture or on a dedicated detection thread. The planner chooses based on latency vs simplicity tradeoff.
- **D-09:** Detection processes each captured frame from the latest-frame-wins handoff. Stale frames are dropped, not queued (DET-05).

### Phase 3 UI scope
- **D-10:** Phase 3 adds NO frontend UI. Detection is purely a backend engine. Frontend detection status and alert display belong to Phase 4.
- **D-11:** Tauri detection events MAY be emitted to the frontend in Phase 3 for future Phase 4 consumption, but the frontend does not render them in this phase.

### Threshold configuration
- **D-12:** The existing `ColorMatchConfig` fields (`hsv_lower`, `hsv_upper`, `min_pixels`, `min_ratio`) are the configuration surface. Phase 3 does not add new threshold controls.
- **D-13:** Threshold validation ensures `min_pixels > 0`, `min_ratio` in (0.0, 1.0], and `hsv_lower <= hsv_upper` per channel. Validation errors use Chinese messages consistent with Phase 1/2 patterns.

### Claude's Discretion
- Exact Rust module names and file structure for the detection engine, as long as they follow the existing `src-tauri/src/` conventions.
- Exact detection thread model (inline with capture vs separate thread), as long as latency stays bounded.
- Exact `DetectionResult` struct field names, as long as it carries matched-rule info, pixel counts, ratios, ROI id, and timestamp.
- Whether to emit Tauri events for detection results in Phase 3 or defer to Phase 4.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` §Phase 3 — Official Phase 3 goal, success criteria, and three plan slots: HSV pipeline, threshold config/validation, latest-frame-wins behavior and tests.
- `.planning/REQUIREMENTS.md` §Detection — Requirement IDs `DET-01` through `DET-05` define the observable outcomes for this phase.
- `.planning/PROJECT.md` §Target features, §Constraints — HSV color matching with configurable ranges, OpenCV as eventual but not immediate dependency.

### Prior locked decisions
- `.planning/phases/01-foundation-and-config-spine/01-CONTEXT.md` §DPI and coordinate contract — Physical pixels are the internal system of record.
- `.planning/phases/01-foundation-and-config-spine/01-CONTEXT.md` §Configuration lifecycle — Monitoring uses a frozen config copy; no hot config updates.
- `.planning/phases/02-mss-capture-loop/02-CONTEXT.md` §Frame delivery contract — Latest-frame-wins semantics, bounded handoff, frame metadata (ROI id, physical-pixel region, timestamp, dimensions, RGBA buffer).
- `.planning/phases/02-mss-capture-loop/02-CONTEXT.md` §Capture cadence and runtime lifecycle — `std::thread` for blocking work, clean start/stop.
- `.planning/phases/02-mss-capture-loop/02-SUMMARY.md` — Confirms `CapturedFrame`, `MonitoringStatus`, `MssCaptureWorker`, and validation helpers are available.

### Architecture and research
- `.planning/research/ARCHITECTURE.md` §Capture → Detect → Alert Pipeline — Shows detection as the middle stage between capture and alerts.
- `.planning/research/STACK.md` §Detection — Notes OpenCV as the eventual detection backend but acknowledges pure-Rust alternatives for HSV matching.
- `.planning/research/PITFALLS.md` §Frame Processing Latency — Warns about unbounded frame queues and stale-frame accumulation.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/models/config.rs` — `ColorMatchConfig` with `hsv_lower: [u32; 3]`, `hsv_upper: [u32; 3]`, `min_pixels: u32`, `min_ratio: f64`. `default_hostile_marker()` returns H:0-15, S:120-255, V:120-255. `RoiConfig.color_rules: Vec<ColorMatchConfig>`.
- `src-tauri/src/capture/mss.rs` — `CapturedFrame` struct with RGBA buffer (`raw_data: Vec<u8>`), `roi_id`, `region: Rect`, `captured_at`, `width`, `height`. `MssCaptureWorker` with latest-frame-wins `Arc<Mutex<Option<CapturedFrame>>>`.
- `src-tauri/src/commands/monitoring.rs` — `MonitoringController` with managed state, `MonitoringSnapshot`, event emission (`monitoring-status`, `monitoring-error`).
- `src-tauri/src/dpi/contract.rs` — Physical-pixel coordinate types and conversion helpers.

### Established Patterns
- Worker threads use `Arc<AtomicBool>` for cancellation and `JoinHandle` for clean shutdown.
- Shared state uses `Arc<Mutex<T>>` with latest-frame-wins semantics.
- Error messages in Chinese for user-facing validation failures.
- Tauri events pushed to frontend via `app_handle.emit()`.

### Integration Points
- Detection engine reads `CapturedFrame` from the capture worker's latest-frame-wins slot.
- Detection results feed into the monitoring lifecycle (Phase 4 will wire alerts).
- `ColorMatchConfig` rules from frozen `MonitorConfig.rois[].color_rules` drive detection thresholds.

</code_context>

<specifics>
## Specific Ideas

- User explicitly chose pure Rust HSV implementation over OpenCV for Phase 3.
- User explicitly chose OR logic for multi-rule evaluation: any color rule match triggers detection.
- User explicitly chose no frontend UI for Phase 3; detection is a backend-only engine.

</specifics>

<deferred>
## Deferred Ideas

- OpenCV integration for HSV/template matching — deferred to Phase 8 or post-v1.0 (DETX-01).
- Debug HSV mask image dumps — `DebugConfig.dump_hsv_masks` field exists but implementation deferred to Phase 8 (DBG-01).
- Detection status display in frontend — deferred to Phase 4 (alert pipeline).
- Morphology-based noise filtering — deferred to v2 (DETX-02).

</deferred>

---

*Phase: 03-hsv-detection-engine*
*Context gathered: 2026-04-28*
