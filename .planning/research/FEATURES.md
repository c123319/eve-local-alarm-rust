# Features Research — EVE Local Alert v1.0

## Feature Categories

### 1. Screen Capture

**Table stakes:**
- **MSS desktop capture**: Select a screen region, capture frames at regular intervals. Window must remain visible. Simple, reliable, low overhead.
- **WGC multi-window capture**: Each EVE client window captured independently via Windows Graphics Capture API. Supports background/minimized windows. Each window gets its own capture thread.

**Differentiators:**
- **Auto-detect EVE windows**: Enumerate windows with "EVE" in the title, present as capture targets automatically.
- **Adaptive frame rate**: Reduce capture rate when no changes detected, increase when changes occur.

**Complexity:** Medium. WGC requires window handle enumeration, per-window pipeline management. MSS is simpler.

**Dependencies:** None — capture is the first step in the pipeline.

### 2. Detection (HSV Color Match)

**Table stakes:**
- **Configurable HSV ranges**: User can tune H/S/V min/max for red and orange standings markers.
- **min_pixels threshold**: Minimum number of matching pixels to trigger a positive detection.
- **min_ratio threshold**: Minimum ratio of matching pixels to total ROI area.
- **Real-time processing**: Process each captured frame through HSV filter.

**Differentiators:**
- **Multiple color rules**: Support several color match configs (red hostile, orange neutral, etc.)
- **Visual preview**: Show HSV mask overlay in real-time during configuration.

**Complexity:** Medium. OpenCV `inRange` + `countNonZero` is straightforward. Tuning HSV ranges is the hard part (game lighting, anti-aliasing).

**Dependencies:** Capture pipeline must provide frames.

**Edge cases:**
- DPI scaling affects pixel coordinates and capture dimensions
- EVE UI overlay transparency can interfere with color detection
- Different EVE graphics settings (low/medium/high) change marker rendering
- Window focus changes can affect WGC capture behavior

### 3. Alerts

**Table stakes:**
- **Popup notification**: In-app popup with hostile count, auto-close after configurable timeout.
- **Sound playback**: Play a WAV/MP3 file on detection. Configurable sound file path.
- **Windows Toast**: Native Windows notification. Works even when app is minimized.

**Differentiators:**
- **Cooldown per ROI**: After alerting, don't re-alert for the same ROI for N seconds.
- **Alert escalation**: Different sounds for different threat levels.

**Complexity:** Low-Medium. Sound and Toast are single-function APIs. Popup is React UI.

**Dependencies:** Detection results must trigger alerts.

**Edge cases:**
- Sound file not found or unsupported format
- Multiple alerts firing simultaneously (audio overlap)
- Toast notification rate limiting by Windows
- Popup appearing while user is in ROI selector mode

### 4. ROI Selector

**Table stakes:**
- **Interactive region selection**: Click-drag to define capture region on screen.
- **Real-time preview**: Show captured region content in the selector.
- **Hit box overlay**: Show detection results overlaid on the preview.

**Differentiators:**
- **Preset regions**: Common EVE local chat positions (left panel, right panel).
- **Snap-to-window**: Auto-detect EVE window boundaries.

**Complexity:** High. Requires screen capture integration, React canvas/SVG overlay, real-time updates.

**Dependencies:** Capture pipeline for preview, detection for hit box overlay.

### 5. Configuration

**Table stakes:**
- **JSON save/load**: Persist all settings to a JSON file in AppData.
- **Runtime freeze**: Deep copy config at startup so detection uses stable settings.
- **Default config**: Ship with sensible defaults for EVE Online.

**Complexity:** Low. Serde handles serialization. `clone()` for deep copy.

**Dependencies:** All features depend on config for their settings.

### 6. System Tray

**Table stakes:**
- **Minimize to tray**: Hide window instead of closing.
- **Quick start/stop**: Toggle monitoring from tray menu.
- **Status indicator**: Show monitoring status in tray icon (green = active, red = alert).

**Complexity:** Low-Medium. Tauri v2 built-in tray API.

**Dependencies:** Monitoring state management.

### 7. Debug Mode

**Table stakes:**
- **HSV mask dumps**: Save the binary mask image after HSV filtering to disk.
- **Detection overlays**: Save captured frame with detection bounding boxes.

**Complexity:** Low. File I/O with OpenCV `imwrite`.

**Dependencies:** Detection pipeline.

## Feature Dependencies (Build Order)

```
Config → Capture → Detection → Alerts
                  ↘ ROI Selector (needs Capture + Detection preview)
System Tray (independent)
Debug Mode (extends Detection)
```

## Recommended Build Priority

1. Config model + JSON persistence
2. MSS capture (simpler, proves the pipeline)
3. HSV detection
4. Alerts (popup + sound + toast)
5. ROI selector
6. WGC multi-window capture
7. System tray
8. Debug mode
