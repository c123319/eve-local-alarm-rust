---
phase: 02-mss-capture-loop
created: 2026-04-27
source: 02-RESEARCH.md
---

# Phase 02 Validation Strategy

## Validation Architecture

### Must-haves

1. `CAP-01`: User can monitor a single visible screen region through MSS-style desktop capture.
2. `UI-02`: User can start and stop monitoring from the main app UI.
3. Stop lifecycle is clean: repeated start/stop does not leave orphaned capture workers.
4. Captured frames flow into a stable Rust-side contract for Phase 3 detection.

### Automated evidence

- `cargo check --manifest-path src-tauri/Cargo.toml` exits 0.
- `cargo test --manifest-path src-tauri/Cargo.toml capture` exits 0.
- `cargo test --manifest-path src-tauri/Cargo.toml monitoring` exits 0.
- `npm run build` exits 0.

### Static evidence

- `src-tauri/Cargo.toml` contains `xcap`.
- `src-tauri/src/capture/mss.rs` contains `capture_region`.
- `src-tauri/src/commands/monitoring.rs` contains `start_monitoring`, `stop_monitoring`, and `get_monitoring_status`.
- `src/components/SettingsPanel.tsx` contains `监控控制`, `开始监控`, `停止监控`, and `MSS 模式仅捕获屏幕可见区域`.
