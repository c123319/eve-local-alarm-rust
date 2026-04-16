# Research Summary — EVE Local Alert v1.0

## Key Findings

### Stack Additions
- **Capture**: `windows-capture` (WGC multi-window) + `xcap` (MSS desktop region)
- **Detection**: `opencv` crate (OpenCV C++ bindings, requires system OpenCV 4.x)
- **Alerts**: `tauri-plugin-notification` (Toast) + `rodio` (sound) + React popup (Tauri events)
- **Tray**: Tauri v2 built-in `tray-icon` feature (no extra crate)
- **Config**: `serde` + `serde_json` (standard Rust serialization)
- **Frontend**: React 18 + TypeScript + Vite + `@tauri-apps/api` v2

### Feature Table Stakes
1. **Capture**: Both WGC and MSS modes are essential for feature parity. WGC for multi-boxing, MSS for simplicity.
2. **Detection**: HSV color matching with configurable ranges is the proven approach from the Python version. min_pixels + min_ratio thresholds for noise filtering.
3. **Alerts**: All three types (popup, sound, toast) needed. Per-ROI cooldown is critical to prevent alert spam.
4. **ROI Selector**: Interactive selection with real-time preview is high-complexity but essential for usability.
5. **Config**: JSON + runtime freeze prevents mid-detection config mutation issues.

### Build Order (Dependencies)
```
Config model → MSS capture → HSV detection → Alerts → ROI Selector → WGC capture → System Tray → Debug Mode
```

### Watch Out For
1. **DPI scaling** — Get coordinates right from phase 1 or everything breaks
2. **OpenCV build setup** — System library dependency can be a blocker; document and resolve early
3. **Alert spam** — Cooldown + throttle from the start, not bolted on later
4. **Config freeze** — Deep copy at monitoring start, never read live config during detection
5. **WGC permissions** — Graceful fallback to MSS if WGC fails for a window
6. **Tauri event volume** — Throttle events to frontend, don't emit every frame
