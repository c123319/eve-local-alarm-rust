---
phase: 3
slug: hsv-detection-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | None — uses Cargo.toml test profile |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml detection` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml detection`
- **After every plan wave:** Run `cargo test --manifest-path src-tauri/Cargo.toml`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | DET-01, DET-04 | — | N/A | unit | `cargo test --manifest-path src-tauri/Cargo.toml evaluate_frame` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | DET-01 | — | N/A | unit | `cargo test --manifest-path src-tauri/Cargo.toml hsv_conversion` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 1 | DET-01, DET-02, DET-03 | T-03-01 | validate_color_match_config rejects out-of-range HSV bounds and thresholds | unit | `cargo test --manifest-path src-tauri/Cargo.toml validate_color_match` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 2 | DET-05 | — | N/A | unit | `cargo test --manifest-path src-tauri/Cargo.toml latest_frame_wins` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/detection/mod.rs` — module declaration and re-exports
- [ ] `src-tauri/src/detection/hsv.rs` — RGB->HSV conversion unit tests (known RGB->HSV pairs)
- [ ] `src-tauri/src/detection/engine.rs` — DetectionEngine and evaluate_frame tests (synthetic frames)
- [ ] `src-tauri/src/detection/validation.rs` — validate_color_match_config tests
- [ ] `src-tauri/src/lib.rs` — add `mod detection;` declaration

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None | — | All phase behaviors have automated verification | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
