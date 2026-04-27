# Phase 2: MSS Capture Loop - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `02-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 2 - MSS Capture Loop
**Areas discussed:** MSS capture backend, capture cadence, thread model, frame delivery, Tauri lifecycle commands, Phase 2 UI scope

---

## MSS capture backend

| Option | Description | Selected |
|--------|-------------|----------|
| Direct Windows GDI API | Manual Windows-only screenshot implementation using native APIs | |
| XCap | Rust crate selected in project research for MSS-style desktop capture | ✓ |
| Other screenshot crate | Alternative crates such as `screenshots` | |

**User's choice:** Initially chose direct Windows GDI API, then changed decision to **XCap as MSS screenshot backend**.
**Notes:** The final context treats GDI as superseded. Downstream agents should plan around `xcap` unless implementation proves it unusable.

---

## Capture cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcoded FPS | Simpler but less flexible | |
| Configurable frame rate | User/runtime can tune CPU vs latency | ✓ |
| Adaptive cadence | More complex; can react to processing time | |

**User's choice:** 可配置的帧率.
**Notes:** Context recommends conservative defaults around 5-10 FPS for local chat monitoring.

---

## Thread model

| Option | Description | Selected |
|--------|-------------|----------|
| `std::thread` | Dedicated blocking capture worker | ✓ |
| Tokio async task | Better for async IO, less natural for blocking capture | |
| Tauri async runtime only | Would require careful blocking isolation | |

**User's choice:** `std::thread`.
**Notes:** Capture is blocking work, so the final context keeps the worker off the Tauri command path.

---

## Frame delivery

| Option | Description | Selected |
|--------|-------------|----------|
| Unbounded queue | Simple but risks stale frame buildup | |
| Latest-frame-wins bounded handoff | Drops stale frames and protects latency | ✓ |
| Frontend preview stream first | Useful later, not required for Phase 2 | |

**User's choice:** Delegated to implementation defaults.
**Notes:** Claude selected latest-frame-wins because Phase 3 explicitly requires bounded latency and stale-frame dropping.

---

## Tauri lifecycle commands

| Option | Description | Selected |
|--------|-------------|----------|
| Single `start_monitoring` / `stop_monitoring` pair | Clear lifecycle API matching research docs | ✓ |
| Multi-step setup/start commands | More flexible but unnecessary for this phase | |
| Frontend-only state simulation | Does not satisfy capture lifecycle requirements | |

**User's choice:** Delegated to implementation defaults.
**Notes:** Context locks explicit start/stop commands and event-based status reporting.

---

## Phase 2 UI scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal controls in existing settings UI | Fastest path to satisfy UI-02 | ✓ |
| New monitoring screen | More structure but premature for Phase 2 | |
| Full ROI selector | Out of scope until Phase 5 | |

**User's choice:** Delegated to implementation defaults.
**Notes:** Context keeps UI utilitarian and Chinese-first, consistent with Phase 1.

---

## Claude's Discretion

- Exact Rust module names and status payload shape.
- Exact bounded channel implementation.
- Exact conservative default FPS within the 5-10 FPS range.

## Deferred Ideas

- Direct Windows GDI capture implementation — superseded by XCap.
- HSV detection — Phase 3.
- Alerts — Phase 4.
- ROI selector and preview — Phase 5.
- WGC multi-window capture — Phase 6.
