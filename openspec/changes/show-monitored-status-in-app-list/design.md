## Context

The current selector already merges discovered apps/processes and supports filtering, but each row only reflects discovery/runtime presence (for example running vs installed). Since target sessions now stay alive until explicit pause, the selector needs to expose per-target monitoring lifecycle state so users can tell whether a row represents an active monitor, a paused monitor, or a target not yet monitored in the current session.

## Goals / Non-Goals

**Goals:**
- Show per-target monitoring status directly in the app selection list.
- Reuse existing in-memory session state (`active` / `paused`) as the source of truth.
- Keep selector filtering, sorting, and enter-to-open behavior unchanged.

**Non-Goals:**
- No new keybindings or monitor lifecycle behavior changes.
- No persistence-format changes for history/session caches.
- No redesign of selector layout beyond adding monitoring-state text.

## Decisions

1. Add an optional monitor-state field to selector row data.
- Decision: Extend `AppInfo` with an optional selector-facing monitoring state enum/string.
- Rationale: `AppSelector` renders from `AppInfo`; adding state there keeps rendering logic local and avoids separate side-channel maps.
- Alternative considered: Keep a parallel `HashMap<MonitorTarget, State>` inside `AppSelector`. Rejected because it duplicates target derivation and increases sync risk.

2. Compute selector monitor state in main TUI flow before launching selector.
- Decision: When building the merged app list, map each row to `MonitorTarget`, check session registry, and annotate row as `Monitoring`, `Paused`, or `Not monitored`.
- Rationale: Main loop already owns session registry and target conversion helpers, making this the most reliable place to enrich row data.
- Alternative considered: Query sessions from inside `AppSelector`. Rejected because `AppSelector` should remain a pure UI component over provided data.

3. Render monitor state as a dedicated compact label in each selector row.
- Decision: Add a stable text segment (fixed-width or clearly delimited) so users can scan monitoring status without ambiguity.
- Rationale: Inline state labels minimize navigation friction and preserve current list interaction model.
- Alternative considered: Color-only indicator. Rejected because color-only encoding is weaker for accessibility and harder to parse in low-color terminals.

## Risks / Trade-offs

- [Row width pressure on narrow terminals] -> Mitigation: keep labels short and trim long app names as currently done.
- [State mismatch if list is not refreshed after session updates] -> Mitigation: selector tick callback already drains session updates; row state is recomputed each time selector is reopened.
- [Ambiguous state naming] -> Mitigation: use explicit labels (`MONITORING`, `PAUSED`, `IDLE`/`NOT MONITORED`) and keep wording consistent with monitor view status.

## Migration Plan

1. Introduce monitor-state representation on selector row model (`AppInfo` and related conversions).
2. Annotate app list entries with session-derived state before creating `AppSelector`.
3. Update selector row rendering and tests/snapshots to include monitor-state label.
4. Verify unchanged behavior for filtering, selection, and existing monitor resume flow.

Rollback: remove the new row field and rendering segment; selector returns to previous runtime/install-only status display.

## Open Questions

- Use `IDLE` vs `NOT MONITORED` for the default state label. This implementation will prefer `IDLE` if width is constrained.
