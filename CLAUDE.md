# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EVE Local Alert is a Windows desktop application that monitors EVE Online's "Local" chat member list and raises alerts when hostile (red/standings) markers are detected. This is a full rewrite of an existing Python/PyQt5 application into **Rust + Tauri v2** (React + TypeScript frontend).

**Current state**: Pre-implementation. Architecture and requirements are defined in `.planning/PROJECT.md`.

## Tech Stack

- **Backend**: Rust (Tauri v2)
- **Frontend**: React + TypeScript (Tauri webview)
- **Screen capture**: Windows Graphics Capture (WGC) via `windows-capture` crate (multi-window), `xcap` or similar for MSS-style desktop capture (single ROI)
- **Detection**: OpenCV via `opencv-rust` bindings — HSV color matching + template matching
- **Alerts**: Windows Toast notifications, popup with auto-close/cooldown, sound playback
- **Platform**: Windows 10/11 only (WGC API dependency)

## Architecture

Two capture modes run independent pipelines:

- **WGC mode**: Each EVE client window gets its own capture → detect pipeline. Scales for multi-boxing. Windows are enumerated, each with dedicated capture and detection.
- **MSS mode**: Simpler desktop region capture. Window must remain visible on screen.

Detection pipeline per ROI: HSV color match (configurable ranges, min_pixels/min_ratio) + template match (OpenCV matchTemplate, multi-template, scale search). Results combined with debounce/cooldown logic.

### Config Model

- `MonitorConfig` — global settings
- `TargetConfig` — per WGC window
- `RoiConfig` — per ROI region
- `ColorMatchConfig` — per color rule (HSV ranges)
- `TemplateMatchConfig` — per template rule

Config saved/loaded as JSON with runtime freeze (deep copy on start).

## Build & Run (once implemented)

```bash
# Install frontend dependencies
cd frontend && npm install

# Dev mode (hot reload)
cargo tauri dev

# Build release
cargo tauri build

# Run tests
cargo test

# Run single test
cargo test test_name

# Lint
cargo clippy
cargo fmt --check
```

## Domain Context

EVE Online's "Local" channel shows all pilots in the same solar system. Hostile pilots appear with red/orange standings markers next to their names. The tool captures the local member list screen region and detects these colored markers to warn the user.

- UI language is **Chinese** by default, with i18n-ready architecture from the start.
- DingTalk Webhook is **out of scope** — replaced by Windows Toast notifications.
- 始终用中文回答和交互

## Planning

This project uses GSD (Get Stuff Done) workflow. Planning artifacts live in `.planning/`. Project requirements, decisions, and evolution tracking are in `.planning/PROJECT.md`.
