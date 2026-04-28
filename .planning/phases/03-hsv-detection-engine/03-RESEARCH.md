# Phase 3: HSV Detection Engine - Research

**Researched:** 2026-04-28
**Domain:** Pure-Rust HSV color matching detection pipeline for EVE Online hostile marker detection
**Confidence:** HIGH

## Summary

Phase 3 builds the HSV color-matching detection engine that converts captured RGBA frames (from Phase 2's `CapturedFrame`) into structured `DetectionResult` outcomes. The key locked decision (D-01) is to implement HSV conversion and pixel matching entirely in pure Rust without OpenCV -- this is straightforward because RGB-to-HSV conversion is a compact ~20-line math function, and pixel iteration over a flat RGBA byte buffer is idiomatic Rust.

The existing codebase already provides the foundation: `CapturedFrame` carries an `rgba: Vec<u8>` buffer with width/height metadata, `ColorMatchConfig` defines HSV bounds as `[u32; 3]` in OpenCV half-range convention (H: 0-179, S: 0-255, V: 0-255), and `MssCaptureWorker` uses latest-frame-wins semantics via `Arc<Mutex<Option<CapturedFrame>>>`. The detection engine plugs into this pipeline by consuming frames from the capture worker's latest-frame slot, evaluating all color rules with OR logic, and producing structured results.

No new crate dependencies are required. The RGB-to-HSV algorithm is self-contained math. The existing `Arc<Mutex<Option<T>>>` pattern and `AtomicBool` cancellation from Phase 2 carry forward directly to the detection thread model.

**Primary recommendation:** Implement detection as a pure-Rust pixel iterator with no external HSV crate. Create a `detection` module under `src-tauri/src/` with `DetectionResult`, `DetectionEngine`, and validation helpers. Wire the engine into the capture pipeline using the established latest-frame-wins + `std::thread` + `AtomicBool` patterns.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use pure Rust pixel iteration for HSV color matching. Do not introduce OpenCV in Phase 3. The detection engine iterates the RGBA frame buffer, converts each pixel to HSV, and compares against configurable bounds.
- **D-02:** OpenCV introduction is deferred to Phase 8 (debug/diagnostics) or later for template matching (DETX-01). Phase 3 must not depend on OpenCV build infrastructure.
- **D-03:** HSV conversion uses the standard RGB->HSV formula. Hue range is 0-179 (OpenCV convention half-range, matching the existing `hsv_lower`/`hsv_upper` defaults of [0,120,120] to [15,255,255]).
- **D-04:** Multiple `ColorMatchConfig` rules within a single ROI use OR logic: if ANY rule matches, the frame counts as a positive detection.
- **D-05:** A rule matches when the count of in-range pixels meets BOTH `min_pixels` AND `min_ratio` thresholds (conjunction within a single rule).
- **D-06:** Each frame evaluation produces a structured `DetectionResult` containing: whether detection is positive, which rule(s) matched, matched pixel count and ratio per rule, ROI id, and frame timestamp.
- **D-07:** Detection results are delivered through a latest-frame-wins channel, consistent with the capture frame delivery pattern.
- **D-08:** Detection runs on the same thread as capture or on a dedicated detection thread. The planner chooses based on latency vs simplicity tradeoff.
- **D-09:** Detection processes each captured frame from the latest-frame-wins handoff. Stale frames are dropped, not queued (DET-05).
- **D-10:** Phase 3 adds NO frontend UI. Detection is purely a backend engine. Frontend detection status and alert display belong to Phase 4.
- **D-11:** Tauri detection events MAY be emitted to the frontend in Phase 3 for future Phase 4 consumption, but the frontend does not render them in this phase.
- **D-12:** The existing `ColorMatchConfig` fields (`hsv_lower`, `hsv_upper`, `min_pixels`, `min_ratio`) are the configuration surface. Phase 3 does not add new threshold controls.
- **D-13:** Threshold validation ensures `min_pixels > 0`, `min_ratio` in (0.0, 1.0], and `hsv_lower <= hsv_upper` per channel. Validation errors use Chinese messages consistent with Phase 1/2 patterns.

### Claude's Discretion
- Exact Rust module names and file structure for the detection engine, as long as they follow the existing `src-tauri/src/` conventions.
- Exact detection thread model (inline with capture vs separate thread), as long as latency stays bounded.
- Exact `DetectionResult` struct field names, as long as it carries matched-rule info, pixel counts, ratios, ROI id, and timestamp.
- Whether to emit Tauri events for detection results in Phase 3 or defer to Phase 4.

### Deferred Ideas (OUT OF SCOPE)
- OpenCV integration for HSV/template matching -- deferred to Phase 8 or post-v1.0 (DETX-01).
- Debug HSV mask image dumps -- `DebugConfig.dump_hsv_masks` field exists but implementation deferred to Phase 8 (DBG-01).
- Detection status display in frontend -- deferred to Phase 4 (alert pipeline).
- Morphology-based noise filtering -- deferred to v2 (DETX-02).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DET-01 | User can configure HSV lower/upper bounds for hostile marker detection. | Already implemented: `ColorMatchConfig.hsv_lower`/`hsv_upper` in `config.rs` with defaults [0,120,120]-[15,255,255]. Phase 3 adds validation (D-13). |
| DET-02 | User can configure `min_pixels` threshold for a positive detection. | Already implemented: `ColorMatchConfig.min_pixels` (default 12). Phase 3 adds validation `> 0` (D-13). |
| DET-03 | User can configure `min_ratio` threshold for a positive detection. | Already implemented: `ColorMatchConfig.min_ratio` (default 0.02). Phase 3 adds validation `(0.0, 1.0]` (D-13). |
| DET-04 | App evaluates each captured frame with HSV color matching and emits structured detection results. | Core detection engine: RGB->HSV conversion, pixel counting, OR-logic multi-rule evaluation, structured `DetectionResult` (D-06). |
| DET-05 | Detection pipeline uses a latest-frame-wins strategy so processing latency does not grow unbounded during monitoring. | Follows existing `Arc<Mutex<Option<T>>>` pattern from Phase 2 capture worker. Detection consumes latest frame, drops stale ones (D-07, D-09). |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| HSV conversion & pixel iteration | API / Backend (Rust) | -- | Pure math, no UI involvement |
| Detection result struct | API / Backend (Rust) | -- | Data model for internal pipeline |
| Threshold validation | API / Backend (Rust) | -- | Input validation before detection runs |
| Latest-frame-wins detection pipeline | API / Backend (Rust) | -- | Thread/channel coordination, same pattern as capture |
| Detection event emission | Frontend Server (Tauri) | -- | `app_handle.emit()` for future Phase 4 consumption |
| Detection status display | -- | -- | DEFERRED to Phase 4 |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` + `serde_json` | 1.x (existing) | `DetectionResult` serialization for Tauri events | Already in project, derive pattern established |
| `std::thread` | Rust stdlib | Detection thread lifecycle | Phase 2 established pattern: `AtomicBool` + `JoinHandle` |
| `std::sync::{Arc, Mutex, AtomicBool}` | Rust stdlib | Shared state, cancellation, latest-frame-wins | Phase 2 established pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri::Emitter` | 2.x (existing) | Event emission for detection results | Optional in Phase 3, required in Phase 4 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled RGB->HSV | `hsv` crate (0.1.1) or `color_space` crate | Unnecessary dependency for ~20 lines of math. User explicitly chose pure Rust (D-01). Hand-rolled is fully sufficient and avoids crate churn. |
| Separate detection thread | Inline detection in capture thread | Inline is simpler but couples capture timing to detection latency. Separate thread keeps capture cadence independent (D-08 gives planner discretion). |
| `crossbeam-channel` bounded(1) | `Arc<Mutex<Option<T>>>` | Channel approach is more idiomatic for producer-consumer but existing pattern is already established in Phase 2 and well-understood. Consistency wins. |

**Installation:**
```bash
# No new dependencies required for Phase 3
# All functionality uses Rust stdlib + existing serde/tauri crates
```

**Version verification:** All dependencies already present in `src-tauri/Cargo.toml`. No new crates needed. Verified by reading Cargo.toml: tauri 2, serde 1, serde_json 1, xcap 0.9, dirs 5.0.

## Architecture Patterns

### System Architecture Diagram

```
Capture Worker (Phase 2)                 Detection Engine (Phase 3)              Future: Phase 4
========================                ==============================          ===============

                              latest_frame
  XCap capture  ─────────►  Arc<Mutex<Option<CapturedFrame>>>
  (per ROI)                  (latest-frame-wins)         │
                                                         │ clone()
                                                         ▼
                                                 ┌──────────────────┐
                                                 │ DetectionEngine  │
                                                 │                  │
                                                 │ 1. Read RGBA buf │
                                                 │ 2. Per-rule:     │
                                                 │    pixel iter    │
                                                 │    RGB->HSV      │
                                                 │    in-range check│
                                                 │    count + ratio │
                                                 │ 3. OR across     │
                                                 │    rules          │
                                                 │ 4. Build result  │
                                                 └────────┬─────────┘
                                                          │
                                                          ▼
                                                  DetectionResult
                                                  (latest-frame-wins)
                                                          │
                                              ┌───────────┼───────────┐
                                              │           │           │
                                              ▼           ▼           ▼
                                    Arc<Mutex<Option<DetectionResult>>>
                                    (for Phase 4 alert consumer)
                                              │
                                              ▼ (optional in Phase 3)
                                    app_handle.emit("detection-result", ...)
                                              │
                                     Phase 4: AlertManager
                                     consumes DetectionResult
```

### Recommended Project Structure
```
src-tauri/src/
├── detection/                  # NEW: Detection engine module
│   ├── mod.rs                  # Module exports
│   ├── engine.rs               # DetectionEngine struct, evaluate_frame logic
│   ├── hsv.rs                  # RGB->HSV conversion, pixel iteration helpers
│   └── validation.rs           # ColorMatchConfig threshold validation
├── capture/                    # EXISTING: Phase 2 capture module
│   ├── mod.rs
│   └── mss.rs                  # CapturedFrame, MssCaptureWorker
├── commands/                   # EXISTING: Tauri commands
│   ├── mod.rs                  # May add detection wiring
│   ├── config.rs
│   └── monitoring.rs           # MonitoringController (wire detection here)
├── models/                     # EXISTING: Config structs
│   ├── mod.rs
│   └── config.rs               # ColorMatchConfig (validation added here)
├── dpi/                        # EXISTING: DPI handling
├── store/                      # EXISTING: Config persistence
└── lib.rs                      # Module declarations, event constants
```

### Pattern 1: RGB to HSV Conversion (Pure Rust)
**What:** Convert an 8-bit RGBA pixel to HSV in OpenCV half-range convention.
**When to use:** Every pixel during frame evaluation.
**Example:**
```rust
/// Convert an RGBA pixel to HSV using OpenCV half-range convention.
/// Input: r, g, b in [0, 255].
/// Output: (h, s, v) where h in [0, 179], s in [0, 255], v in [0, 255].
///
/// Source: [ASSUMED] based on standard RGB->HSV algorithm and OpenCV convention.
/// OpenCV docs confirm: H: [0,179], S: [0,255], V: [0,255] for uint8 images.
/// Reference: https://docs.opencv.org/4.x/df/d9d/tutorial_py_colorspaces.html
#[inline]
fn rgba_pixel_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Value
    let v = max;

    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Hue (0-360 degrees)
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 4.0)
    } else {
        60.0 * (((r - g) / delta) + 2.0)
    };

    // Normalize to OpenCV convention: H/2 -> [0,179], S*255, V*255
    let h_norm = if h < 0.0 { h + 360.0 } else { h };
    (
        (h_norm / 2.0) as u8,  // H: 0-179
        (s * 255.0) as u8,      // S: 0-255
        (v * 255.0) as u8,      // V: 0-255
    )
}
```

### Pattern 2: Frame Evaluation with Multi-Rule OR Logic
**What:** Evaluate all color rules against a captured frame, return structured result.
**When to use:** Each frame from the capture pipeline.
**Example:**
```rust
/// Evaluate a single captured frame against all color rules for the ROI.
///
/// OR logic (D-04): if ANY rule matches, detection is positive.
/// AND logic within rule (D-05): both min_pixels AND min_ratio must be met.
fn evaluate_frame(
    frame: &CapturedFrame,
    color_rules: &[ColorMatchConfig],
) -> DetectionResult {
    let total_pixels = frame.width as usize * frame.height as usize;
    let mut rule_results: Vec<RuleMatchResult> = Vec::new();
    let mut any_matched = false;

    for rule in color_rules {
        let (matched_count, h_lower, h_upper, s_lower, s_upper, v_lower, v_upper) =
            count_matching_pixels(&frame.rgba, rule);

        let ratio = matched_count as f64 / total_pixels as f64;
        let pixel_threshold_met = matched_count >= rule.min_pixels as usize;
        let ratio_threshold_met = ratio >= rule.min_ratio;
        let rule_matched = pixel_threshold_met && ratio_threshold_met;

        if rule_matched {
            any_matched = true;
        }

        rule_results.push(RuleMatchResult {
            rule_name: rule.name.clone(),
            matched: rule_matched,
            pixel_count: matched_count as u32,
            ratio,
        });
    }

    DetectionResult {
        roi_id: frame.roi_id.clone(),
        detected: any_matched,
        rule_results,
        frame_timestamp_ms: frame.captured_at_ms,
        evaluated_at_ms: now_millis(),
    }
}
```

### Pattern 3: Pixel Iteration and In-Range Counting
**What:** Count pixels within HSV bounds in a flat RGBA buffer.
**When to use:** Core inner loop of detection.
**Example:**
```rust
/// Count pixels in the RGBA buffer that fall within the HSV bounds.
/// RGBA buffer is laid out as [R, G, B, A, R, G, B, A, ...] (4 bytes per pixel).
fn count_matching_pixels(rgba: &[u8], rule: &ColorMatchConfig) -> usize {
    let [h_lo, s_lo, v_lo] = rule.hsv_lower;
    let [h_hi, s_hi, v_hi] = rule.hsv_upper;

    rgba.chunks_exact(4)
        .filter(|pixel| {
            let (h, s, v) = rgba_pixel_to_hsv(pixel[0], pixel[1], pixel[2]);
            h >= h_lo as u8 && h <= h_hi as u8
                && s >= s_lo as u8 && s <= s_hi as u8
                && v >= v_lo as u8 && v <= v_hi as u8
        })
        .count()
}
```

### Pattern 4: Detection Thread Lifecycle (mirrors MssCaptureWorker)
**What:** A detection thread that consumes latest frames and produces detection results.
**When to use:** When detection runs on a separate thread from capture.
**Example:**
```rust
pub struct DetectionWorker {
    cancellation: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    latest_result: Arc<Mutex<Option<DetectionResult>>>,
}

impl DetectionWorker {
    pub fn new(
        frame_source: Arc<Mutex<Option<CapturedFrame>>>,
        color_rules: Vec<ColorMatchConfig>,
    ) -> Self { /* ... */ }

    pub fn start(&mut self) -> Result<(), String> { /* ... */ }
    pub fn stop(&mut self) -> Result<(), String> { /* ... */ }
    pub fn get_latest_result(&self) -> Option<DetectionResult> { /* ... */ }
}
```

### Anti-Patterns to Avoid
- **Pre-allocating HSV conversion buffer:** Do NOT allocate a separate Vec for the HSV image. Iterate RGBA directly and count in-range pixels on-the-fly. This avoids a full frame-sized allocation per detection cycle. Memory usage stays O(1) beyond the existing RGBA buffer.
- **Queue-based frame pipeline:** Do NOT use `mpsc` channels or Vec queues between capture and detection. This violates DET-05 (latest-frame-wins) and creates unbounded latency under load. Use `Arc<Mutex<Option<T>>>` single-slot pattern.
- **Detecting on every pixel including alpha=0:** Skip fully transparent pixels (alpha == 0) to avoid false matches on padding or uninitialized memory. Though in practice XCap returns fully opaque pixels, defensive coding here costs nothing.
- **Using f64 for pixel-level math:** f32 is sufficient for HSV conversion and avoids unnecessary precision overhead. The config thresholds are f64 (from `ColorMatchConfig`), but pixel math benefits from f32 throughput.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thread lifecycle | Custom thread management | Phase 2 pattern: `AtomicBool` + `JoinHandle` + `Arc<Mutex<Option<T>>>` | Already established, tested, handles poison |
| Error message i18n | English error strings | Chinese messages per Phase 1/2 convention | User-facing consistency across all phases |
| Config validation | Ad-hoc validation | `validate_*` helper functions returning `Result<T, String>` | Phase 2 pattern: `validate_capture_fps`, `validate_roi` |
| Frame metadata | Custom frame wrapper | Existing `CapturedFrame` struct from `capture::mss` | Already carries `roi_id`, `captured_at_ms`, `width`, `height`, `rgba` |

**Key insight:** Phase 3 introduces zero new crate dependencies. The detection engine is pure Rust math + iteration over existing data structures. The only "new" code is the HSV conversion formula and the multi-rule evaluation loop.

## Runtime State Inventory

> This is a greenfield module addition, not a rename/refactor phase. No runtime state migration required.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None -- detection is stateless per frame | No migration |
| Live service config | None -- reads from frozen config | No migration |
| OS-registered state | None | No migration |
| Secrets/env vars | None | No migration |
| Build artifacts | `Cargo.toml` unchanged, no new native deps | No rebuild |

## Common Pitfalls

### Pitfall 1: Hue Wrapping in HSV Comparison
**What goes wrong:** Red hues wrap around 0 in the standard H:0-179 range. A range like [170, 10] (red-to-red across the boundary) would fail a simple `h >= lo && h <= hi` check.
**Why it happens:** The standard HSV hue circle is continuous, but the numeric range wraps at 0/179.
**How to avoid:** For Phase 3, the existing default hostile-marker range [0, 15] does NOT wrap, so simple comparison works. If future ranges wrap, the planner must add wrap-aware logic. Document this as a limitation in code comments.
**Warning signs:** A user configures `hsv_lower: [170, ...]` and `hsv_upper: [10, ...]` -- detection would never match.

### Pitfall 2: Integer Overflow in Pixel Counting
**What goes wrong:** `width * height` can overflow `u32` for very large captures (unlikely at 4K but possible).
**Why it happens:** Both `width` and `height` are `u32`; their product needs `usize` cast.
**How to avoid:** Cast to `usize` before multiplication: `(frame.width as usize) * (frame.height as usize)`. This matches how `rgba.len()` is naturally `usize`.
**Warning signs:** Division by zero or incorrect ratio when frame dimensions are unusually large.

### Pitfall 3: min_ratio Calculation with Zero-Size Frames
**What goes wrong:** If `width * height == 0`, the ratio calculation divides by zero.
**Why it happens:** Edge case not caught by validation.
**How to avoid:** ROI validation already rejects zero-width or zero-height (Phase 2 `validate_roi`). Add a defensive early return in `evaluate_frame` if `total_pixels == 0`.
**Warning signs:** NaN or infinity in ratio values.

### Pitfall 4: Poisoned Mutex on Detection Result
**What goes wrong:** If a panic occurs during detection, the `Mutex<Option<DetectionResult>>` becomes poisoned.
**Why it happens:** Phase 2 already handles this pattern with `poisoned.into_inner()`.
**How to avoid:** Follow Phase 2 pattern: match on `lock()` result, use `into_inner()` on poison. Already established in `MssCaptureWorker::get_latest_frame()`.
**Warning signs:** Detection stops producing results after a transient error.

### Pitfall 5: RGBA Buffer Size Mismatch
**What goes wrong:** `rgba.len() != width * height * 4` causes incorrect pixel iteration.
**Why it happens:** XCap returns consistent data, but defensive coding is important.
**How to avoid:** Add a debug assertion at the start of `evaluate_frame`: `assert_eq!(frame.rgba.len(), frame.width as usize * frame.height as usize * 4)`. In release builds, use a checked early return instead.
**Warning signs:** Detection results vary wildly or count is off by small amounts.

## Code Examples

### Complete HSV In-Range Check (Verified)
```rust
// Source: Standard RGB->HSV algorithm + OpenCV uint8 convention
// [VERIFIED: docs.opencv.org/4.x/df/d9d/tutorial_py_colorspaces.html]
// OpenCV confirms: H: [0,179], S: [0,255], V: [0,255] for uint8 images.

/// Check if a single RGBA pixel falls within the given HSV bounds.
/// Bounds use OpenCV half-range convention: H in [0,179], S in [0,255], V in [0,255].
#[inline]
fn pixel_in_hsv_range(r: u8, g: u8, b: u8, lower: &[u32; 3], upper: &[u32; 3]) -> bool {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    // Hue calculation (0-360 degrees, then /2 for OpenCV convention)
    let h_deg = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * (((bf - rf) / delta) + 4.0)
    } else {
        60.0 * (((rf - gf) / delta) + 2.0)
    };
    let h = ((if h_deg < 0.0 { h_deg + 360.0 } else { h_deg }) / 2.0) as u8;

    // Saturation and Value
    let s = if max == 0.0 { 0u8 } else { (delta / max * 255.0) as u8 };
    let v = (max * 255.0) as u8;

    // In-range check against bounds (lower and upper are [u32; 3])
    h >= lower[0] as u8 && h <= upper[0] as u8
        && s >= lower[1] as u8 && s <= upper[1] as u8
        && v >= lower[2] as u8 && v <= upper[2] as u8
}
```

### Validation Helper (Chinese messages)
```rust
// Source: Follows Phase 2 pattern: validate_capture_fps, validate_roi
// [VERIFIED: src-tauri/src/capture/mss.rs lines 226-241]

/// Validate a ColorMatchConfig's threshold fields.
/// Returns Ok(()) or Err with Chinese message.
pub fn validate_color_match_config(config: &ColorMatchConfig) -> Result<(), String> {
    if config.min_pixels == 0 {
        return Err("最小像素数必须大于 0".to_string());
    }
    if config.min_ratio <= 0.0 || config.min_ratio > 1.0 {
        return Err("最小像素比例必须在 (0.0, 1.0] 范围内".to_string());
    }
    for ch in 0..3 {
        if config.hsv_lower[ch] > config.hsv_upper[ch] {
            return Err(format!(
                "HSV 下界不能大于上界 (通道 {}): {} > {}",
                ch, config.hsv_lower[ch], config.hsv_upper[ch]
            ));
        }
    }
    Ok(())
}
```

### DetectionResult Struct
```rust
// Source: D-06 from CONTEXT.md
// [VERIFIED: .planning/phases/03-hsv-detection-engine/03-CONTEXT.md]

/// Result of evaluating a single frame against color rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMatchResult {
    /// Name of the color rule that was evaluated
    pub rule_name: String,
    /// Whether this specific rule matched
    pub matched: bool,
    /// Number of pixels that fell within HSV bounds
    pub pixel_count: u32,
    /// Ratio of matching pixels to total pixels
    pub ratio: f64,
}

/// Complete detection result for one frame evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionResult {
    /// ROI ID from the captured frame
    pub roi_id: String,
    /// Whether any color rule matched (OR logic)
    pub detected: bool,
    /// Per-rule evaluation results
    pub rule_results: Vec<RuleMatchResult>,
    /// Timestamp of the source frame (ms since epoch)
    pub frame_timestamp_ms: u128,
    /// Timestamp when evaluation completed (ms since epoch)
    pub evaluated_at_ms: u128,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| OpenCV-dependent HSV detection | Pure Rust pixel iteration | Phase 3 decision (D-01) | No native library dependency, simpler build, faster iteration |
| Full HSV image allocation | On-the-fly pixel counting | Design choice | O(1) extra memory per detection cycle |
| Queue-based frame pipeline | Latest-frame-wins single slot | Phase 2 established | Bounded latency, no memory growth |

**Deprecated/outdated:**
- Using the `hsv` crate (0.1.1): Unnecessary for this use case. The conversion is ~20 lines of math. Adding a crate for this adds supply chain risk with no benefit.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | RGBA buffer from XCap is always [R, G, B, A] with 4 bytes per pixel | Code Examples | Pixel iteration produces wrong HSV values. Verified by reading `capture_frame()` in mss.rs which uses `image.as_raw().to_vec()` -- XCap returns RGBA8. [VERIFIED by code inspection] |
| A2 | Default hostile marker range [0,120,120]-[15,255,255] correctly detects red markers | Code Examples | Detection misses hostile markers. This default was set in Phase 1 based on the original Python app's proven values. [ASSUMED -- ported from working Python version] |
| A3 | Hue wrapping (e.g., range [170, 10]) is not needed for Phase 3 | Common Pitfalls | If users configure wrapping hue ranges, detection fails silently. The existing default does not wrap. Future enhancement needed. [ASSUMED -- default range is [0,15], no wrap] |
| A4 | f32 precision is sufficient for HSV conversion | Anti-Patterns | Slight mismatch in edge-case pixels. Standard practice in real-time detection systems. [ASSUMED] |
| A5 | XCap always returns fully opaque pixels (alpha = 255) | Anti-Patterns | If alpha pixels are present, they get processed. No harm -- alpha is ignored in HSV conversion. [ASSUMED] |

**Assumptions needing confirmation:** A2 (default HSV range correctness) should be validated during Phase 8 debug mode with real EVE screenshots. A3 (no hue wrapping) should be documented as a known limitation.

## Open Questions

1. **Detection thread model: inline vs separate thread?**
   - What we know: D-08 allows either. Inline is simpler (fewer Arc/Mutex, no extra thread). Separate thread keeps capture cadence independent of detection latency.
   - What's unclear: At 5 FPS default with small ROIs (~200x100 pixels), detection is very fast (<1ms estimated), so inline may be acceptable.
   - Recommendation: Start with inline detection inside the capture loop. If profiling shows detection latency affecting capture cadence, refactor to a separate thread. The `DetectionEngine` API should be thread-agnostic to allow this refactoring.

2. **Should Phase 3 emit Tauri detection events?**
   - What we know: D-11 says "MAY emit." Phase 4 needs them for alerts.
   - What's unclear: Whether emitting events without a frontend listener causes any overhead.
   - Recommendation: Emit events in Phase 3. Tauri event emission with no listener is a no-op (very cheap). This makes Phase 4 integration trivial and allows manual testing via devtools.

3. **How to handle the `DetectionResult` handoff to Phase 4?**
   - What we know: D-07 says latest-frame-wins. But alert logic in Phase 4 needs to know about EVERY positive detection (not just the latest).
   - What's unclear: Whether Phase 4 alert cooldown tracking can work with latest-frame-wins results.
   - Recommendation: Emit every detection result as a Tauri event (not just latest). The `latest_result` field on the worker is for status queries, not the primary alert feed. Phase 4 will consume events, not poll latest_result.

## Environment Availability

> Phase 3 has no new external dependencies. All functionality uses Rust stdlib + existing project crates.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | Yes | stable MSVC | -- |
| serde + serde_json | Serialization | Yes | 1.x | -- |
| tauri | Event emission | Yes | 2.x | -- |
| cargo test | Testing | Yes | -- | -- |
| cargo clippy | Linting | Yes | -- | -- |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | None -- uses Cargo.toml test profile |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml detection` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DET-01 | HSV bounds configurable and validated | unit | `cargo test --manifest-path src-tauri/Cargo.toml validate_color_match` | Wave 0 |
| DET-02 | min_pixels threshold validated (>0) | unit | `cargo test --manifest-path src-tauri/Cargo.toml validate_color_match` | Wave 0 |
| DET-03 | min_ratio threshold validated ((0,1]) | unit | `cargo test --manifest-path src-tauri/Cargo.toml validate_color_match` | Wave 0 |
| DET-04 | Frame evaluation produces structured DetectionResult | unit | `cargo test --manifest-path src-tauri/Cargo.toml evaluate_frame` | Wave 0 |
| DET-05 | Latest-frame-wins drops stale frames | unit | `cargo test --manifest-path src-tauri/Cargo.toml latest_frame_wins` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml detection`
- **Per wave merge:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Phase gate:** Full suite green (currently 30 tests passing) + all new detection tests green.

### Wave 0 Gaps
- [ ] `src-tauri/src/detection/mod.rs` -- module declaration and re-exports
- [ ] `src-tauri/src/detection/hsv.rs` -- RGB->HSV conversion unit tests (known RGB->HSV pairs)
- [ ] `src-tauri/src/detection/engine.rs` -- DetectionEngine and evaluate_frame tests (synthetic frames)
- [ ] `src-tauri/src/detection/validation.rs` -- validate_color_match_config tests
- [ ] `src-tauri/src/lib.rs` -- add `mod detection;` declaration

## Security Domain

> Phase 3 processes image data locally with no network, authentication, or user input beyond config values. Security surface is minimal.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No auth in this phase |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | No access control |
| V5 Input Validation | yes | `validate_color_match_config` validates threshold ranges |
| V6 Cryptography | no | No crypto |

### Known Threat Patterns for Pure Rust Detection

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious config values | Tampering | Validation rejects out-of-range HSV bounds and thresholds |
| Integer overflow in pixel counting | Denial of Service | Cast to usize before multiplication; early return on zero-area frames |
| Panic in detection thread | Denial of Service | Use `Result<T, String>` error handling, not unwrap(); handle Mutex poison |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/03-hsv-detection-engine/03-CONTEXT.md` -- All locked decisions D-01 through D-13
- `src-tauri/src/capture/mss.rs` -- CapturedFrame struct, MssCaptureWorker, latest-frame-wins pattern [VERIFIED by code inspection]
- `src-tauri/src/models/config.rs` -- ColorMatchConfig struct, default_hostile_marker() [VERIFIED by code inspection]
- `src-tauri/src/commands/monitoring.rs` -- MonitoringController pattern [VERIFIED by code inspection]
- `src-tauri/Cargo.toml` -- Existing dependencies [VERIFIED by file read]
- OpenCV official docs -- HSV range convention H:[0,179], S:[0,255], V:[0,255] [CITED: https://docs.opencv.org/4.x/df/d9d/tutorial_py_colorspaces.html]

### Secondary (MEDIUM confidence)
- Stack Overflow -- OpenCV HSV range explanation [CITED: https://stackoverflow.com/questions/10948589/]
- Wikipedia -- Standard RGB to HSV algorithm [ASSUMED as standard knowledge, no specific URL needed]

### Tertiary (LOW confidence)
- Performance characteristics of pixel iteration at scale -- [ASSUMED based on general Rust knowledge, not benchmarked for this specific ROI size]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Zero new dependencies; pure Rust stdlib + existing crates. All patterns verified by code inspection.
- Architecture: HIGH -- Follows established Phase 2 patterns exactly (Arc<Mutex<Option<T>>>, AtomicBool cancellation, thread lifecycle).
- Pitfalls: HIGH -- Based on verified OpenCV convention docs and direct code inspection of existing capture/config modules.
- HSV algorithm: HIGH -- Standard mathematical conversion, verified against OpenCV convention docs.

**Research date:** 2026-04-28
**Valid until:** 2026-05-28 (stable -- no fast-moving dependencies)
