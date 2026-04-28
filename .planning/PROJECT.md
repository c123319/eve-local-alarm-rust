# EVE Local Alert (Rust + Tauri)

## What This Is

EVE Local Alert is a Windows desktop application that monitors EVE Online's "Local" chat member list region and raises alerts when hostile (red/standings) markers are detected. This is a full rewrite of an existing Python/PyQt5 application into Rust + Tauri (React + TypeScript frontend), aiming for better performance, smaller binary size, and a modern UI while maintaining full feature parity.

## Core Value

Detect hostile markers in EVE Online local chat regions and alert the user immediately. If detection or alerting fails, the tool is useless.

## Current Milestone: v1.0 Core Alert Pipeline

**Goal:** Ship the complete capture → detect → alert loop with both capture modes, interactive ROI setup, and all alert types.

**Target features:**
- MSS desktop region capture + WGC multi-window capture
- HSV color match detection (configurable ranges, min_pixels/min_ratio)
- Popup notification with auto-close + cooldown
- Sound playback (configurable sound file)
- Windows Toast notification
- Interactive ROI selector with real-time preview + hit box overlay
- JSON config save/load with runtime freeze
- System tray (minimize-to-tray, quick start/stop)
- Debug mode with HSV mask image dumps
- Per-ROI debounce/cooldown
- Chinese UI (no i18n framework)

## Requirements

### Validated

- [x] Configuration save/load to JSON with runtime freeze (deep copy on start) — Validated in Phase 01
- [x] Capture screen regions via MSS-style desktop capture for single ROI monitoring — Validated in Phase 02

### Active

- [ ] Capture screen regions via Windows Graphics Capture (WGC) API for multi-window monitoring
- [ ] Detect hostile markers using HSV color matching (configurable HSV ranges with min_pixels/min_ratio thresholds)
- [ ] Alert via popup notification with auto-close and cooldown
- [ ] Alert via sound playback (configurable sound file)
- [ ] Alert via Windows Toast notification
- [ ] ROI selector with real-time preview and hit box overlay
- [ ] Multi-window monitoring with per-window capture + detect pipelines
- [ ] System tray support (minimize-to-tray, quick start/stop)
- [ ] Per-ROI debounce/cooldown
- [ ] Debug mode with image dumps for HSV masks
- [ ] Chinese UI

### Future (deferred from v1.0)

- Detect hostile markers using OpenCV template matching (multi-template, configurable threshold, scale search)
- Per-window debounce/cooldown overrides
- i18n-ready architecture
- Morphology filtering (min/max area, min/max width-height) for detection noise suppression

### Out of Scope

- DingTalk Webhook — replaced by Windows Toast notification in this rewrite
- Cross-platform support — Windows 10/11 only (WGC dependency)
- Mobile companion app — not relevant to desktop monitoring tool
- Cloud sync of configs — local-only tool

## Context

- **Original project**: Python/PyQt5 (`eve_local_alert_project`), uses PyQt5 for UI, OpenCV for detection, WGC/MSS for capture, shared memory IPC between processes
- **Why rewrite**: Better performance (Rust vs Python), smaller binary, native feel via Tauri, modern UI stack (React + TS), eliminate Python runtime dependency
- **Key domain knowledge**: EVE Online "Local" channel shows all pilots in the same solar system. Hostile pilots appear with red/orange standings markers. The tool captures the local member list region and detects these colored markers.
- **WGC mode**: Each EVE window gets its own capture + detect pipeline, communicating via shared memory. Scales linearly for multi-boxing.
- **MSS mode**: Simpler, captures a desktop region. Window must remain visible.
- **Detection pipeline**: HSV color match (configurable ranges) + template match (OpenCV matchTemplate). Results combined per ROI with debounce logic.
- **Config model**: `MonitorConfig` contains global settings, `TargetConfig` per WGC window, `RoiConfig` per ROI region, `ColorMatchConfig` per color rule, `TemplateMatchConfig` per template rule.

## Constraints

- **Platform**: Windows 10/11 only (WGC API requirement)
- **Tech stack**: Rust backend (Tauri) + React + TypeScript frontend
- **Capture**: WGC via `windows-capture` crate, MSS via `xcap` or similar Rust crate
- **Detection**: OpenCV via `opencv-rust` or custom HSV/template matching in Rust
- **Runtime**: Tauri v2 for native webview + Rust command system
- **Language**: Chinese default, i18n architecture from the start

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Tauri instead of Python/PyQt5 | Performance, binary size, modern UI, no Python runtime needed | — Pending |
| React + TypeScript frontend | Rich ecosystem, good for complex interactive UI (ROI selector, multi-window table) | — Pending |
| Drop DingTalk Webhook | Simplify alert system, Windows Toast is native and more universal | — Pending |
| Both WGC + MSS capture | Preserve full feature parity, WGC for multi-window, MSS for simplicity | — Pending |
| OpenCV via Rust bindings | Reuse proven detection algorithms, avoid reimplementation | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-28 — Phase 02 complete (MSS capture + monitoring lifecycle)*
