# Phase 02: MSS Capture Loop - Pattern Map

**Created:** 2026-04-27
**Status:** Ready for planner/executor

## Files to create or modify

| Target | Role | Closest existing analog | Pattern to follow |
|--------|------|-------------------------|-------------------|
| `src-tauri/src/capture/mod.rs` | Capture module exports | `src-tauri/src/dpi/mod.rs`, `src-tauri/src/store/mod.rs` | Small module file that re-exports public types/functions |
| `src-tauri/src/capture/mss.rs` | MSS capture service | `src-tauri/src/dpi/contract.rs` | Focused Rust module with structs, pure helpers, and unit tests |
| `src-tauri/src/commands/monitoring.rs` | Tauri lifecycle commands | `src-tauri/src/commands/config.rs`, `src-tauri/src/commands/dpi.rs` | `#[tauri::command] pub async fn ... -> Result<_, String>` |
| `src-tauri/src/commands/mod.rs` | Command re-exports | Existing file | Add `pub mod monitoring;` and command re-exports |
| `src-tauri/src/models/config.rs` | Capture FPS config | Existing defaults in `MonitorConfig`, `RoiConfig` | Add serde-default field with conservative default |
| `src-tauri/src/lib.rs` | App state/events/handler registration | Existing command registration and `events` module | Register monitoring commands, manage state, define event constants |
| `src/components/SettingsPanel.tsx` | Monitoring controls | Existing config/DPI sections | Add Chinese section with invoke/listen/error state |
| `src-tauri/Cargo.toml` | Native dependency | Existing dependencies block | Add `xcap` dependency |

## Existing code patterns

### Tauri command registration

Current pattern in `src-tauri/src/lib.rs`:

```rust
use commands::{save_config, load_config, get_default_config, get_config_status, get_dpi_info, validate_roi_coordinates};

.invoke_handler(tauri::generate_handler![
    save_config,
    load_config,
    get_default_config,
    get_config_status,
    get_dpi_info,
    validate_roi_coordinates,
])
```

Phase 2 should extend this with `start_monitoring`, `stop_monitoring`, and `get_monitoring_status`.

### Command module re-export

Current pattern in `src-tauri/src/commands/mod.rs`:

```rust
pub mod config;
pub mod dpi;

pub use config::save_config;
pub use dpi::validate_roi_coordinates;
```

Phase 2 should add `pub mod monitoring;` and re-export monitoring commands.

### Config model defaults

`MonitorConfig`, `RoiConfig`, and nested structs use `#[serde(default)]` and custom `Default` implementations. Phase 2 config additions must keep backwards-compatible JSON loading.

### Frontend invoke pattern

`SettingsPanel.tsx` uses:

```tsx
const savedConfig = await invoke<MonitorConfig>('load_config');
await invoke('save_config', { config });
```

Phase 2 should use `invoke('start_monitoring', { config })`, `invoke('stop_monitoring')`, and `invoke<MonitoringStatus>('get_monitoring_status')`.

## Planning implications

- Split backend capture/service and frontend lifecycle wiring into two plans matching ROADMAP plan slots.
- Backend plan must establish all Rust types/commands/events before the UI plan consumes them.
- UI plan depends on backend command/event names and should not invent new backend names.

## PATTERN MAPPING COMPLETE
