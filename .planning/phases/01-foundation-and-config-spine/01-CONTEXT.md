# Phase 1: Foundation and Config Spine - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 delivers the buildable Tauri v2 application skeleton, the stable config model and persistence behavior, the OpenCV/native dependency contract for development machines, and the DPI coordinate contract that later capture and ROI phases must follow. It does not implement real capture, detection, alerts, or full ROI tooling yet.

</domain>

<decisions>
## Implementation Decisions

### Configuration lifecycle
- **D-01:** App startup auto-loads the last saved config, but does not auto-start monitoring.
- **D-02:** v1.0 uses a single fixed primary config file rather than multiple named profiles.
- **D-03:** Monitoring does not support hot config updates; config changes only take effect after monitoring is stopped and restarted.
- **D-04:** If the config file is missing, corrupted, or cannot be parsed, the app falls back to defaults and shows a clear user-facing warning.

### OpenCV and native dependency strategy
- **D-05:** Phase 1 optimizes for reliable developer-machine builds first, not full dependency automation or distribution completeness.
- **D-06:** OpenCV setup may rely on manual installation plus environment-variable configuration in v1.0.
- **D-07:** The project should lock to a clearly documented recommended OpenCV 4.x version range instead of trying to support all possible installations.
- **D-08:** Missing or misconfigured OpenCV should fail clearly at build time with actionable setup guidance, rather than deferring failure to runtime.

### DPI and coordinate contract
- **D-09:** Internal system-of-record coordinates use physical pixels only.
- **D-10:** ROI interactions are performed in user-visible display space, then converted immediately into physical-pixel coordinates for storage and runtime use.
- **D-11:** DPI scaling changes, display resolution changes, or moving a window between displays with different DPI should mark the affected ROI as potentially invalid.
- **D-12:** Potential DPI invalidation is tracked per ROI, not at whole-config or whole-app granularity.
- **D-13:** Users may continue monitoring with a potentially invalid ROI, but the app must show a clear risk warning.
- **D-14:** The warning behavior is two-layered: show a one-time warning when monitoring starts, and keep a persistent yellow/red warning indicator visible in the UI afterward.

### Phase 1 product shape
- **D-15:** Phase 1 must ship a usable Chinese-language settings experience, not just an internal scaffold.
- **D-16:** The Phase 1 UI only covers Phase 1 concerns (scaffold/config/environment/DPI contracts) and should not prematurely expose later-phase capture, detection, alert, or WGC configuration surfaces.
- **D-17:** The Phase 1 UI should be plain and utilitarian rather than highly polished or design-forward.
- **D-18:** The landing view for Phase 1 should treat configuration status and environment/dependency health as equally important primary information.

### Claude's Discretion
- Exact folder/file naming inside the app's config storage area, as long as the single-config-file model stays intact.
- Exact presentation of the utilitarian Chinese settings UI, as long as config status and environment health are both first-class.
- Exact wording, severity colors, and iconography of DPI risk warnings, as long as startup warning + persistent warning are both preserved.
- Exact implementation approach for build-time OpenCV checks and setup documentation, as long as failures happen early and guidance is clear.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and milestone intent
- `.planning/PROJECT.md` — Project purpose, v1.0 milestone goal, active requirements, and platform constraints.
- `.planning/ROADMAP.md` §Phase 1 — The official scope anchor, success criteria, and plan slots for this phase.
- `.planning/REQUIREMENTS.md` §Configuration, §Platform and Reliability — Requirement IDs CONF-01..CONF-04 and PLAT-01..PLAT-02 mapped to Phase 1.

### Architecture and delivery order
- `.planning/research/SUMMARY.md` — Locked build order and top implementation risks across the milestone.
- `.planning/research/ARCHITECTURE.md` §Key Architecture Decisions, §Suggested Build Order — System boundaries, config/runtime model, and sequencing rationale.

### Dependency and build constraints
- `.planning/research/STACK.md` §Detection, §Build Dependencies — OpenCV requirement, native dependency expectations, and crate-level guardrails.
- `.planning/research/PITFALLS.md` §1 DPI Scaling Issues, §2 OpenCV Rust Binding Build Complexity, §5 Config Mutation During Active Detection — The primary failure modes this phase must design against.

### Feature and boundary mapping
- `.planning/research/FEATURES.md` §Configuration, §Feature Dependencies, §Recommended Build Priority — Phase-1-relevant feature categories and why config precedes capture/detection.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- No application source files exist yet; this phase starts from planning artifacts only.

### Established Patterns
- Planning docs already establish a strict GSD structure: requirements → roadmap → phase context → plan files.
- Research documents consistently prioritize reliability over convenience for early milestone work, especially around DPI, OpenCV, and config immutability.

### Integration Points
- Phase 1 will create the initial Rust/Tauri frontend/backend structure that all later capture and detection work will attach to.
- The config model created here becomes the contract for later MSS, WGC, detection, alert, and debug phases.
- The DPI contract defined here becomes the shared rule for ROI selection, capture coordinates, overlays, and debug artifacts.

</code_context>

<specifics>
## Specific Ideas

- Phase 1 should feel like a practical Chinese-language control panel rather than a polished marketing-style interface.
- The first screen should behave like a base operations console: configuration status and environment health are equally prominent.
- The system should prefer visible warnings over silent auto-correction when DPI conditions make ROI trust questionable.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation-and-config-spine*
*Context gathered: 2026-04-24*
