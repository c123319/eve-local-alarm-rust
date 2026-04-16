# Pitfalls Research — EVE Local Alert v1.0

## Critical Pitfalls

### 1. DPI Scaling Issues (Severity: HIGH)

**Problem:** Windows DPI scaling causes mismatched coordinates between what the user sees (scaled) and what capture APIs return (physical pixels). ROI selection coordinates don't match capture coordinates.

**Warning signs:**
- ROI selector captures wrong region
- Detection works on some monitors but not others
- Coordinates off by exactly 1.25x or 1.5x (common scale factors)

**Prevention:**
- Query DPI awareness at startup, set DPI-aware context
- Use physical (unscaled) coordinates internally for all capture and detection
- Apply DPI scaling only at the display layer (ROI selector UI)
- Test on 125%, 150%, 200% scale displays

**Phase to address:** Phase 1 (project scaffold + config) — get this right from the start.

### 2. OpenCV Rust Binding Build Complexity (Severity: HIGH)

**Problem:** `opencv-rust` requires OpenCV C++ libraries installed on the system. Build fails if `OPENCV_LINK_PATHS` and `OPENCV_INCLUDE_PATHS` are not set correctly. Different OpenCV versions cause API mismatches.

**Warning signs:**
- `cargo build` fails with linker errors
- CI builds break on different machines
- OpenCV version mismatch runtime panics

**Prevention:**
- Document exact OpenCV version requirement in README
- Use `opencv` crate feature flags to match installed OpenCV version
- Consider `opencv` crate's `buildtime-bindgen` feature for version flexibility
- Set up a consistent build environment (vcpkg or manual OpenCV install)

**Phase to address:** Phase 1 (project scaffold) — resolve build setup before any code.

### 3. WGC Capture Permissions & Borderless Windows (Severity: MEDIUM)

**Problem:** Windows Graphics Capture API requires specific permissions and may not capture borderless or fullscreen windows reliably. Some EVE client configurations don't expose a capturable window handle.

**Warning signs:**
- WGC returns blank/black frames
- "Access denied" errors when starting capture
- Capture works for some EVE windows but not others

**Prevention:**
- Fall back gracefully to MSS if WGC fails for a specific window
- Test with both borderless and windowed EVE modes
- Handle WGC API errors without crashing the capture thread
- Implement retry logic with exponential backoff

**Phase to address:** Phase with WGC capture implementation.

### 4. Alert Spam & Cooldown Edge Cases (Severity: MEDIUM)

**Problem:** Without proper debounce, detection fires on every frame with matching pixels, flooding the user with alerts. Edge cases: cooldown timer reset on new detection, multiple ROIs triggering simultaneously.

**Warning signs:**
- User gets 30+ alerts per second
- Sound playback overlaps or stutters
- Toast notifications queue up and overwhelm Windows notification center

**Prevention:**
- Per-ROI cooldown timer with configurable duration
- Global alert throttle (max N alerts per minute across all ROIs)
- Sound playback: cancel previous sound before playing new one (or queue)
- Toast: batch rapid detections into a single notification

**Phase to address:** Alert implementation phase.

### 5. Config Mutation During Active Detection (Severity: MEDIUM)

**Problem:** If user changes config while detection is running, the detection pipeline may see partial config updates (e.g., new HSV range but old min_pixels), causing inconsistent behavior.

**Warning signs:**
- Detection behaves erratically after config changes
- App crashes when config struct is read while being written

**Prevention:**
- Runtime freeze: Deep copy config at monitoring start, detection only reads the frozen copy
- Config changes require stopping and restarting monitoring
- Use `Arc<RwLock<>>` if live config updates are needed later

**Phase to address:** Config model phase — design the freeze pattern from the start.

### 6. Frame Processing Latency (Severity: LOW-MEDIUM)

**Problem:** If OpenCV processing takes longer than the capture interval, frames queue up, causing increasing latency and memory usage.

**Warning signs:**
- Alerts fire seconds after the hostile appeared
- Memory usage grows over time
- Capture frame rate drops below configured rate

**Prevention:**
- Drop frames if previous frame not yet processed (latest-frame-wins strategy)
- Measure and log detection latency in debug mode
- Set reasonable capture rate defaults (5-10 FPS is sufficient for local chat monitoring)

**Phase to address:** Detection pipeline phase.

### 7. ROI Selector UX Complexity (Severity: LOW-MEDIUM)

**Problem:** Building an interactive region selector that works correctly with DPI scaling, multiple monitors, and real-time preview is surprisingly complex.

**Warning signs:**
- Selector region doesn't match captured region
- Preview stutters or shows wrong content
- Selector doesn't work on multi-monitor setups

**Prevention:**
- Use a fullscreen overlay window for selection (Tauri can create transparent fullscreen windows)
- Test on multi-monitor setups early
- Keep preview frame rate low (2-5 FPS) to avoid performance issues
- Consider using native OS screen capture for the selector (not the app's own capture pipeline)

**Phase to address:** ROI selector phase.

### 8. Tauri v2 Event Volume (Severity: LOW)

**Problem:** Emitting too many events from Rust to React (e.g., every detection result, every frame preview) can overwhelm the webview bridge and cause UI lag.

**Warning signs:**
- UI becomes unresponsive during active monitoring
- Frontend console shows event queue warnings
- Alert popup appears delayed

**Prevention:**
- Throttle events sent to frontend (batch detection results)
- Only send frame preview when ROI selector is active
- Use debounced event emission for frequent updates

**Phase to address:** Integration/testing phase.

## Summary: Phase Assignment

| Pitfall | Severity | Phase to Address |
|---------|----------|-----------------|
| DPI scaling | HIGH | Phase 1 (scaffold) |
| OpenCV build complexity | HIGH | Phase 1 (scaffold) |
| WGC permissions | MEDIUM | WGC capture phase |
| Alert spam/cooldown | MEDIUM | Alert phase |
| Config mutation | MEDIUM | Config phase |
| Frame processing latency | LOW-MED | Detection phase |
| ROI selector UX | LOW-MED | ROI selector phase |
| Tauri event volume | LOW | Integration phase |
