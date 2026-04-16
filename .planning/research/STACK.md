# Stack Research — EVE Local Alert v1.0

## Core Framework

| Component | Crate/Library | Version | Rationale |
|-----------|--------------|---------|-----------|
| App shell | `tauri` | 2.x | Native webview, Rust backend, plugin system, small binary |
| Frontend | React + TypeScript | React 18+, TS 5.x | Rich interactive UI for ROI selector, config panels |
| Build tooling | Vite | 5.x+ | Fast HMR, Tauri v2 integrated |

## Screen Capture

| Crate | Purpose | Why |
|-------|---------|-----|
| `windows-capture` | WGC multi-window capture | GPU-accelerated, per-window capture, Windows Graphics Capture API. Best for multi-boxing EVE clients. |
| `xcap` | MSS-style desktop region capture | Cross-platform but Windows-focused. Captures desktop regions without WGC overhead. Window must remain visible. |

**Why both:** WGC for headless/background window capture (multi-client). xcap for simple single-ROI desktop capture.

## Detection

| Crate | Purpose | Why |
|-------|---------|-----|
| `opencv` (opencv-rust) | HSV color matching, image processing | Bindings to OpenCV C++ library. `inRange` for HSV masking, `findContours` for blob detection. Proven detection algorithms from the Python version. |

**Build requirement:** OpenCV 4.x native libraries must be installed on the build system (`OPENCV_LINK_PATHS`, `OPENCV_INCLUDE_PATHS`).

## Notifications & Alerts

| Crate | Purpose | Why |
|-------|---------|-----|
| `tauri-plugin-notification` | Windows Toast notifications | Official Tauri v2 plugin, native OS integration, simple API |
| `rodio` | Sound playback | Most popular Rust audio crate, supports WAV/MP3/OGG. Non-blocking playback via threads. |

**Popup alerts** are implemented in the React frontend (Tauri webview) — no separate crate needed. Use Tauri events to trigger popup state changes.

## System Tray

| Crate | Purpose | Why |
|-------|---------|-----|
| `tauri` (built-in `tray-icon` feature) | System tray icon | Built into Tauri v2 via `TrayIconBuilder`. Menu items, click handlers, icon customization. No extra crate needed. |

Enable in `tauri.conf.json`: `"trayIcon": { "iconPath": "icons/icon.png" }` and Cargo feature `"tray-icon"`.

## Config & Serialization

| Crate | Purpose | Why |
|-------|---------|-----|
| `serde` + `serde_json` | JSON config save/load | De facto standard for Rust serialization. Derive macros for config structs. |
| `dirs` | Config file paths | Cross-platform standard directories (AppData on Windows) |

Runtime freeze: Deep copy via `clone()` on config structs at startup. No special crate needed.

## Frontend Stack

| Library | Purpose |
|---------|---------|
| `react` + `react-dom` | UI framework |
| `typescript` | Type safety |
| `vite` | Build tool |
| `@tauri-apps/api` | Tauri v2 frontend API (invoke, events) |
| `antd` or similar | UI component library for Chinese-friendly design |

## What NOT to Add

- **`notify-rust`**: Redundant with `tauri-plugin-notification`
- **`cpal`**: Too low-level; `rodio` wraps it adequately
- **`image` crate**: OpenCV handles all image processing needs
- **`screenshots` crate**: `xcap` is more capable and maintained

## Build Dependencies

- **OpenCV 4.x**: Must be installed on system, `opencv` crate links against it
- **Rust toolchain**: `rustup`, MSVC toolchain (for Windows)
- **Node.js 18+**: Frontend tooling
- **Windows SDK**: For WGC API, Toast notifications
