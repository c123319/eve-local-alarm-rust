# Phase 3: HSV Detection Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 03-hsv-detection-engine
**Areas discussed:** HSV implementation approach, Multi-rule combination logic, Detection status UI

---

## HSV Implementation Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Pure Rust | Zero dependency, simple build, HSV matching via pixel iteration. OpenCV deferred to Phase 8+. | ✓ |
| OpenCV | Introduce opencv-rust now. Unified HSV/morphology/template matching. | |
| Claude's Discretion | User does not care about specific implementation. | |

**User's choice:** Pure Rust (recommended)
**Notes:** User confirmed pure Rust keeps Phase 3 build simple. OpenCV build complexity is deferred. Default HSV range (H:0-15, S:120-255, V:120-255) is straightforward pure-math.

---

## Multi-rule Combination Logic

| Option | Description | Selected |
|--------|-------------|----------|
| Any-match (OR) | Any color rule match triggers detection. More sensitive, suits EVE hostile markers. | ✓ |
| All-match (AND) | All rules must match. More strict, for compound conditions. | |
| Claude's Discretion | User does not care about specific logic. | |

**User's choice:** Any-match OR (recommended)
**Notes:** Different hostile colors (red, orange, flashing red) are independent threat signals. Any one appearing should trigger.

---

## Detection Status UI

| Option | Description | Selected |
|--------|-------------|----------|
| Backend only | Phase 3 builds engine only. No frontend UI. Detection results feed Phase 4 alerts. | ✓ |
| Minimal status | Show detection FPS and last detection time next to monitoring status. | |
| Claude's Discretion | User does not care about UI presence. | |

**User's choice:** Backend only (recommended)
**Notes:** Phase 3 is purely an engine phase. Frontend detection display belongs to Phase 4 alert pipeline.

---

## Claude's Discretion

- Exact Rust module structure for detection engine
- Detection thread model (inline with capture vs separate thread)
- DetectionResult struct field names
- Whether to emit Tauri detection events in Phase 3 or defer to Phase 4

## Deferred Ideas

- OpenCV integration — Phase 8 or post-v1.0
- Debug HSV mask dumps — Phase 8 (DBG-01)
- Frontend detection status display — Phase 4
- Morphology filtering — v2 (DETX-02)
