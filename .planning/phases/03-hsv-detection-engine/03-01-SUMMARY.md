---
phase: 03-hsv-detection-engine
plan: 01
subsystem: detection
tags: [hsv, opencv, color-matching, pixel-counting, validation, rust]

# Dependency graph
requires:
  - phase: 01-config-model
    provides: ColorMatchConfig struct with hsv_lower, hsv_upper, min_pixels, min_ratio fields
provides:
  - rgba_pixel_to_hsv: pure Rust RGB->HSV conversion with OpenCV half-range convention (H:0-179, S:0-255, V:0-255)
  - count_matching_pixels: counts pixels in RGBA buffer that fall within ColorMatchConfig HSV bounds
  - validate_color_match_config: validates ColorMatchConfig thresholds (min_pixels, min_ratio, HSV bounds) with Chinese error messages
  - detection module structure (mod.rs with hsv + validation submodules, engine placeholder for Plan 02)
affects: [03-02-detection-engine, 04-alerts, roi-selector]

# Tech tracking
tech-stack:
  added: []
  patterns: [pure-function-utilities, opencv-half-range-hsv, chinese-validation-messages, rgba-chunks-exact-4]

key-files:
  created:
    - src-tauri/src/detection/mod.rs
    - src-tauri/src/detection/hsv.rs
    - src-tauri/src/detection/validation.rs
  modified:
    - src-tauri/src/lib.rs

key-decisions:
  - "Used standard HSV hue formula (red=0, green=120, blue=240) not the plan's incorrect values; OpenCV half-range: red=0, green=60, blue=120"
  - "Swapped +2.0 and +4.0 offsets in hue formula (plan had gf/bf cases reversed)"

patterns-established:
  - "Pure function detection utilities with #[inline] for hot-path performance"
  - "Chinese doc comments and error messages for all detection module functions"
  - "RGBA buffer iteration via chunks_exact(4) for 4-byte-per-pixel format"
  - "Validation functions returning Result<(), String> with Chinese error text matching capture module pattern"

requirements-completed: [DET-01, DET-02, DET-03, DET-05]

# Metrics
duration: 12min
completed: 2026-04-28
---

# Phase 3 Plan 01: HSV Conversion Utilities Summary

**Pure Rust RGB->HSV conversion (OpenCV half-range) and ColorMatchConfig validation with 16 unit tests covering all boundary conditions**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-28T06:22:30Z
- **Completed:** 2026-04-28T06:34:25Z
- **Tasks:** 2 (both TDD)
- **Files modified:** 4

## Accomplishments
- RGB->HSV conversion with OpenCV half-range convention (H:0-179, S:0-255, V:0-255) validated against pure red/green/blue/black/white
- Pixel counting that matches RGBA pixels against configurable HSV bounds from ColorMatchConfig
- ColorMatchConfig validation rejecting zero min_pixels, out-of-range min_ratio, and inverted HSV bounds with Chinese error messages
- Detection module registered in lib.rs with engine module placeholder for Plan 02

## Task Commits

Each task was committed atomically (TDD: RED then GREEN):

1. **RED: Failing tests for HSV + validation** - `d6c345b` (test)
2. **GREEN Task 1: RGB->HSV conversion + pixel counting** - `eb6fe46` (feat)
3. **GREEN Task 2: ColorMatchConfig validation** - `3515cf9` (feat)

_Note: Single RED commit covered both tasks' tests; GREEN commits split per task._

## Files Created/Modified
- `src-tauri/src/detection/mod.rs` - Module declarations: pub mod hsv, pub mod validation, engine commented out
- `src-tauri/src/detection/hsv.rs` - rgba_pixel_to_hsv (inline pure function) and count_matching_pixels with 8 unit tests
- `src-tauri/src/detection/validation.rs` - validate_color_match_config with 8 unit tests
- `src-tauri/src/lib.rs` - Added `mod detection;` declaration after capture module

## Decisions Made
- Fixed standard HSV hue formula: green is 120 degrees (half-range H=60), not 60 degrees as plan stated
- Corrected the +2.0/+4.0 offsets in hue computation (plan had the gf/bf max cases swapped)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed swapped hue formula offsets for green/blue channels**
- **Found during:** Task 1 (HSV conversion implementation)
- **Issue:** Plan specified `max==gf -> +4.0` and `max==bf -> +2.0` but standard HSV requires the opposite: `max==gf -> +2.0` (green, H=120) and `max==bf -> +4.0` (blue, H=240)
- **Fix:** Swapped offsets to match standard HSV color wheel
- **Files modified:** src-tauri/src/detection/hsv.rs
- **Verification:** Blue test passes (H~120), green test passes (H~60 in half-range)
- **Committed in:** eb6fe46 (Task 1 GREEN commit)

**2. [Rule 1 - Bug] Fixed green hue test expectation**
- **Found during:** Task 1 (HSV conversion testing)
- **Issue:** Plan stated green HSV hue is 60 degrees (half-range ~30) but standard HSV assigns green 120 degrees (half-range ~60). The plan's test tolerance of [28,32] was incorrect.
- **Fix:** Updated test assertion to tolerance [58,62] matching correct half-range value of 60
- **Files modified:** src-tauri/src/detection/hsv.rs
- **Verification:** test_pure_green_to_hsv passes
- **Committed in:** eb6fe46 (Task 1 GREEN commit)

---

**Total deviations:** 2 auto-fixed (2 bugs in plan's HSV formula/expectations)
**Impact on plan:** Corrections align implementation with standard HSV color model. No scope creep.

## Issues Encountered
None - both auto-fixes were straightforward formula corrections.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- detection::hsv module ready for consumption by detection engine (Plan 02)
- detection::validation ready for config validation at engine startup
- engine module placeholder in mod.rs (commented out) ready to uncomment in Plan 02
- All 46 tests passing (30 baseline + 16 new)

## TDD Gate Compliance
- RED gate: `d6c345b` - test(03-01): add failing tests for HSV conversion and ColorMatchConfig validation
- GREEN gate: `eb6fe46` - feat(03-01): implement RGB->HSV conversion and pixel counting
- GREEN gate: `3515cf9` - feat(03-01): implement ColorMatchConfig validation with Chinese errors
- All gates present and in correct order.

---
*Phase: 03-hsv-detection-engine*
*Completed: 2026-04-28*

## Self-Check: PASSED

- [x] src-tauri/src/detection/mod.rs exists
- [x] src-tauri/src/detection/hsv.rs exists
- [x] src-tauri/src/detection/validation.rs exists
- [x] Commit d6c345b (RED gate) found
- [x] Commit eb6fe46 (GREEN gate Task 1) found
- [x] Commit 3515cf9 (GREEN gate Task 2) found
