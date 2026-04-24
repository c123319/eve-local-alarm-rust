# Roadmap: EVE Local Alert (Rust + Tauri)

## Overview

This roadmap takes the project from planning-only state to a complete Windows desktop hostile-alert pipeline for EVE Local chat. It front-loads environment hardening, config correctness, and DPI safety, then builds the core MSS capture → HSV detection → alert loop before layering ROI UX, multi-window WGC support, tray controls, and debug hardening.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation and Config Spine** - Establish the Tauri scaffold, build environment, config model, and DPI/runtime safety rules. (completed 2026-04-24)
- [ ] **Phase 2: MSS Capture Loop** - Ship a working single-ROI desktop capture loop that can be started and stopped safely.
- [ ] **Phase 3: HSV Detection Engine** - Turn captured frames into structured hostile-detection results with threshold controls.
- [ ] **Phase 4: Alert Pipeline** - Deliver popup, sound, and Windows Toast alerts with cooldown and anti-spam behavior.
- [ ] **Phase 5: ROI Selector and Preview UX** - Add interactive ROI setup with live preview, overlays, and DPI-correct coordinates.
- [ ] **Phase 6: WGC Multi-Window Monitoring** - Extend the pipeline to multiple EVE windows with graceful per-target failure handling.
- [ ] **Phase 7: Tray and Monitoring Controls** - Add tray lifecycle flows and clearer monitoring control/status surfaces.
- [ ] **Phase 8: Debug, Localization, and Milestone Hardening** - Add debug outputs, Chinese UX polish, throttling, and end-to-end validation.

## Phase Details

### Phase 1: Foundation and Config Spine
**Goal**: Create a buildable Tauri v2 project with a stable config model, documented native dependencies, and early protection against DPI/config-mutation pitfalls.
**Depends on**: Nothing (first phase)
**Requirements**: [CONF-01, CONF-02, CONF-03, CONF-04, PLAT-01, PLAT-02]
**Success Criteria** (what must be TRUE):
  1. Developers can build the project with documented Tauri/OpenCV prerequisites on Windows.
  2. User-facing config can be saved, loaded, and cloned into a frozen runtime model.
  3. DPI handling rules are explicit enough that later ROI and capture phases can reuse one coordinate contract.
**Plans**: 3 plans

Plans:
- [x] 01-01: Scaffold Tauri v2 + React + TypeScript app shell and baseline Rust/frontend wiring.
- [x] 01-02: Implement config structs, JSON persistence, defaults, and runtime freeze behavior.
- [x] 01-03: Document Windows/OpenCV build prerequisites and codify DPI coordinate rules.

### Phase 2: MSS Capture Loop
**Goal**: Deliver a working single-region capture loop that can acquire frames from the desktop and expose monitoring lifecycle hooks.
**Depends on**: Phase 1
**Requirements**: [CAP-01, UI-02]
**Success Criteria** (what must be TRUE):
  1. User can start monitoring a visible ROI in MSS mode from the app UI.
  2. User can stop monitoring cleanly without orphaned capture work.
  3. Captured frames are available to downstream detection logic through a stable interface.
**Plans**: 2 plans

Plans:
- [ ] 02-01: Build MSS capture service and frame-delivery contract.
- [ ] 02-02: Wire start/stop monitoring lifecycle between frontend and Rust backend.

### Phase 3: HSV Detection Engine
**Goal**: Convert captured frames into configurable hostile-marker detection results with bounded latency.
**Depends on**: Phase 2
**Requirements**: [DET-01, DET-02, DET-03, DET-04, DET-05]
**Success Criteria** (what must be TRUE):
  1. User can configure HSV bounds, min_pixels, and min_ratio for hostile detection.
  2. Monitoring evaluates each captured frame and produces structured detection outcomes.
  3. Detection latency stays bounded by dropping stale frames instead of queueing indefinitely.
**Plans**: 3 plans

Plans:
- [ ] 03-01: Integrate OpenCV HSV pipeline and typed detection result model.
- [ ] 03-02: Implement threshold configuration, validation, and runtime evaluation logic.
- [ ] 03-03: Add latest-frame-wins behavior and baseline detection tests.

### Phase 4: Alert Pipeline
**Goal**: Turn positive detections into actionable popup, sound, and toast alerts without spamming the user.
**Depends on**: Phase 3
**Requirements**: [ALRT-01, ALRT-02, ALRT-03, ALRT-04, ALRT-05]
**Success Criteria** (what must be TRUE):
  1. User receives popup, sound, and Windows Toast alerts on hostile detection.
  2. Repeated detections from the same ROI respect cooldown and debounce settings.
  3. Rapid detections do not create unusable audio overlap or notification floods.
**Plans**: 3 plans

Plans:
- [ ] 04-01: Implement alert manager and Rust-to-frontend alert event flow.
- [ ] 04-02: Add popup, sound, and Windows Toast integrations.
- [ ] 04-03: Enforce cooldown, debounce, and burst-throttling behavior.

### Phase 5: ROI Selector and Preview UX
**Goal**: Give users an interactive, DPI-correct way to define monitoring regions with live visual feedback.
**Depends on**: Phase 4
**Requirements**: [ROI-01, ROI-02, ROI-03, ROI-04]
**Success Criteria** (what must be TRUE):
  1. User can draw and adjust an ROI interactively.
  2. User sees a live preview of the selected region during setup.
  3. ROI coordinates and overlays remain correct on scaled Windows displays.
**Plans**: 3 plans

Plans:
- [ ] 05-01: Build ROI selection overlay and edit interactions.
- [ ] 05-02: Stream low-FPS preview frames and detection overlays into the selector.
- [ ] 05-03: Validate and harden DPI coordinate normalization for ROI flows.

### Phase 6: WGC Multi-Window Monitoring
**Goal**: Expand the working pipeline to EVE window enumeration and per-window WGC capture with isolated failure handling.
**Depends on**: Phase 5
**Requirements**: [CAP-02, CAP-03, CAP-04, WIN-01, WIN-02]
**Success Criteria** (what must be TRUE):
  1. User can enumerate eligible EVE windows and select WGC monitoring targets.
  2. Each selected window runs an independent capture → detect pipeline.
  3. Failure to start or maintain one WGC target does not crash unrelated monitoring targets.
**Plans**: 3 plans

Plans:
- [ ] 06-01: Implement EVE window discovery and WGC target selection model.
- [ ] 06-02: Add per-window WGC capture workers and detection integration.
- [ ] 06-03: Handle WGC startup/runtime failures with graceful isolation and fallback messaging.

### Phase 7: Tray and Monitoring Controls
**Goal**: Make the application practical for continuous use through tray behavior and clearer monitoring controls.
**Depends on**: Phase 6
**Requirements**: [UI-03, UI-04, UI-05]
**Success Criteria** (what must be TRUE):
  1. User can minimize the app to tray without shutting down monitoring.
  2. User can start or stop monitoring directly from the tray.
  3. Monitoring status is obvious from the UI or tray state.
**Plans**: 2 plans

Plans:
- [ ] 07-01: Add tray icon, menu actions, and background/minimize behavior.
- [ ] 07-02: Surface current monitoring status consistently across tray and app UI.

### Phase 8: Debug, Localization, and Milestone Hardening
**Goal**: Finish the milestone with debug outputs, Chinese UX polish, frontend event throttling, and end-to-end validation of the full alert loop.
**Depends on**: Phase 7
**Requirements**: [UI-01, DBG-01, DBG-02, DBG-03, PLAT-03]
**Success Criteria** (what must be TRUE):
  1. User can enable debug dumps that help explain detection and capture behavior.
  2. Primary UI text is presented in Chinese for the shipped v1.0 experience.
  3. End-to-end monitoring remains responsive under active event traffic and passes milestone validation.
**Plans**: 3 plans

Plans:
- [ ] 08-01: Add debug artifact generation for HSV masks and annotated overlays.
- [ ] 08-02: Polish Chinese UI copy and user-facing monitoring flows.
- [ ] 08-03: Throttle frontend event volume and run full milestone verification.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation and Config Spine | 3/3 | Complete   | 2026-04-24 |
| 2. MSS Capture Loop | 0/2 | Not started | - |
| 3. HSV Detection Engine | 0/3 | Not started | - |
| 4. Alert Pipeline | 0/3 | Not started | - |
| 5. ROI Selector and Preview UX | 0/3 | Not started | - |
| 6. WGC Multi-Window Monitoring | 0/3 | Not started | - |
| 7. Tray and Monitoring Controls | 0/2 | Not started | - |
| 8. Debug, Localization, and Milestone Hardening | 0/3 | Not started | - |
