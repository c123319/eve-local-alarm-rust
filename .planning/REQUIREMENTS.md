# Requirements: EVE Local Alert (Rust + Tauri)

**Defined:** 2026-04-24
**Core Value:** Detect hostile markers in EVE Online local chat regions and alert the user immediately. If detection or alerting fails, the tool is useless.

## v1 Requirements

### Configuration

- [x] **CONF-01**: User can save all monitoring settings to a local JSON config file.
- [x] **CONF-02**: User can load a previously saved JSON config file on app start or on demand.
- [x] **CONF-03**: Monitoring runs against a frozen runtime copy of the config so in-flight detection is not affected by live edits.
- [x] **CONF-04**: App ships with sensible default settings for EVE Local monitoring on Windows.

### Capture

- [ ] **CAP-01**: User can monitor a single visible screen region via MSS-style desktop capture.
- [ ] **CAP-02**: User can monitor multiple EVE client windows via Windows Graphics Capture.
- [ ] **CAP-03**: App can enumerate capturable EVE client windows for WGC target selection.
- [ ] **CAP-04**: App handles WGC capture startup failures gracefully without crashing the monitoring session.

### Detection

- [ ] **DET-01**: User can configure HSV lower/upper bounds for hostile marker detection.
- [ ] **DET-02**: User can configure `min_pixels` threshold for a positive detection.
- [ ] **DET-03**: User can configure `min_ratio` threshold for a positive detection.
- [ ] **DET-04**: App evaluates each captured frame with HSV color matching and emits structured detection results.
- [ ] **DET-05**: Detection pipeline uses a latest-frame-wins strategy so processing latency does not grow unbounded during monitoring.

### Alerts

- [ ] **ALRT-01**: User receives an in-app popup alert when a hostile marker is detected.
- [ ] **ALRT-02**: User hears a configurable alert sound when a hostile marker is detected.
- [ ] **ALRT-03**: User receives a Windows Toast notification when a hostile marker is detected.
- [ ] **ALRT-04**: App enforces per-ROI debounce/cooldown to prevent repeated alert spam for the same target.
- [ ] **ALRT-05**: App throttles overlapping alert bursts so popup, sound, and toast remain usable under repeated detections.

### ROI Selection

- [ ] **ROI-01**: User can interactively draw and edit a monitoring region on screen.
- [ ] **ROI-02**: User sees a low-latency live preview for the selected ROI during setup.
- [ ] **ROI-03**: User sees detection hit-box or equivalent visual overlay in the ROI setup flow.
- [ ] **ROI-04**: ROI coordinates remain correct under Windows DPI scaling.

### Window Monitoring

- [ ] **WIN-01**: Each WGC target window runs through its own capture → detect pipeline.
- [ ] **WIN-02**: Monitoring state is isolated enough that one failed target does not stop unrelated targets.

### UI and Operations

- [ ] **UI-01**: Primary user-facing UI text is Chinese.
- [ ] **UI-02**: User can start and stop monitoring from the main app UI.
- [ ] **UI-03**: App can minimize to tray and continue running in the background.
- [ ] **UI-04**: User can quickly start or stop monitoring from the system tray.
- [ ] **UI-05**: App exposes current monitoring status clearly in the UI or tray state.

### Debug and Diagnostics

- [ ] **DBG-01**: User can enable debug mode to dump HSV mask images to disk.
- [ ] **DBG-02**: User can enable debug mode to dump annotated detection overlay images to disk.
- [ ] **DBG-03**: Debug output includes enough context to troubleshoot DPI, capture, and threshold issues.

### Platform and Reliability

- [x] **PLAT-01**: App runs on Windows 10/11 only and does not promise cross-platform behavior.
- [x] **PLAT-02**: OpenCV build prerequisites are explicit enough that developers can build the project reliably.
- [ ] **PLAT-03**: Rust-to-frontend event traffic is throttled so active monitoring does not freeze the UI.

## v2 Requirements

### Detection Enhancements

- **DETX-01**: App can detect hostile markers using OpenCV template matching with configurable threshold and scale search.
- **DETX-02**: App can apply morphology-based filtering to reduce color-match noise.

### Monitoring Enhancements

- **MONX-01**: User can set per-window debounce/cooldown overrides in addition to per-ROI settings.

### Internationalization

- **I18N-01**: App architecture supports future multi-language UI without rewriting the core interface.

## Out of Scope

| Feature | Reason |
|---------|--------|
| DingTalk Webhook alerts | Replaced by Windows Toast in this rewrite |
| Cross-platform support | WGC dependency and target users are Windows-only |
| Mobile companion app | Not part of the desktop local-monitoring core value |
| Cloud sync of configs | Local-only tool for v1.0 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONF-01 | Phase 1 | Complete |
| CONF-02 | Phase 1 | Complete |
| CONF-03 | Phase 1 | Complete |
| CONF-04 | Phase 1 | Complete |
| PLAT-01 | Phase 1 | Complete |
| PLAT-02 | Phase 1 | Complete |
| CAP-01 | Phase 2 | Pending |
| UI-02 | Phase 2 | Pending |
| DET-01 | Phase 3 | Pending |
| DET-02 | Phase 3 | Pending |
| DET-03 | Phase 3 | Pending |
| DET-04 | Phase 3 | Pending |
| DET-05 | Phase 3 | Pending |
| ALRT-01 | Phase 4 | Pending |
| ALRT-02 | Phase 4 | Pending |
| ALRT-03 | Phase 4 | Pending |
| ALRT-04 | Phase 4 | Pending |
| ALRT-05 | Phase 4 | Pending |
| ROI-01 | Phase 5 | Pending |
| ROI-02 | Phase 5 | Pending |
| ROI-03 | Phase 5 | Pending |
| ROI-04 | Phase 5 | Pending |
| CAP-02 | Phase 6 | Pending |
| CAP-03 | Phase 6 | Pending |
| CAP-04 | Phase 6 | Pending |
| WIN-01 | Phase 6 | Pending |
| WIN-02 | Phase 6 | Pending |
| UI-03 | Phase 7 | Pending |
| UI-04 | Phase 7 | Pending |
| UI-05 | Phase 7 | Pending |
| UI-01 | Phase 8 | Pending |
| DBG-01 | Phase 8 | Pending |
| DBG-02 | Phase 8 | Pending |
| DBG-03 | Phase 8 | Pending |
| PLAT-03 | Phase 8 | Pending |

**Coverage:**
- v1 requirements: 35 total
- Mapped to phases: 35
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-24 after milestone v1.0 initialization*
