## Context

The current TUI lifecycle is single-target and view-coupled: entering a monitor view starts one monitor loop, and leaving via `Esc` tears it down immediately. This behavior conflicts with multi-app workflows where users compare traffic across apps in one CLI session. The proposal requires decoupling monitoring lifecycle from screen navigation and adding explicit per-target pause semantics.

## Goals / Non-Goals

**Goals:**
- Keep a selected target's monitoring loop running after the user leaves that target's monitor screen.
- Allow multiple selected targets to remain monitored concurrently during one TUI session.
- Stop monitoring only when the user explicitly presses `p` in that target's monitor screen.
- Preserve existing history persistence behavior so collected addresses remain associated with each target.

**Non-Goals:**
- No cross-process daemonization; persistence across `net-use` process exit is still file-based history only.
- No redesign of selector UI layout beyond minimal state signaling needed for this behavior.
- No changes to collector sampling interval, masking rules, export format, or clipboard format.

## Decisions

1. Introduce a target session registry in TUI main loop.
- Decision: Maintain a `HashMap<MonitorTarget, MonitorSession>` for the entire TUI run instead of creating/shutting a loop per screen visit.
- Rationale: It keeps target monitor lifetime independent from navigation and enables concurrent background collection.
- Alternative considered: Reusing the existing single-loop flow and restarting monitors on each revisit. Rejected because it cannot satisfy "keep monitoring after leaving screen".

2. Separate background collection state from active view rendering.
- Decision: Each session owns long-lived monitor task + latest aggregated snapshot; monitor views bind/unbind to a target session when entered/exited.
- Rationale: Data keeps updating even when no view is attached, and returning to a target immediately shows up-to-date state.
- Alternative considered: Keep direct `mpsc::UnboundedReceiver` ownership in `MonitorView`. Rejected because receiver ownership prevents unattended background draining.

3. Redefine monitor actions for navigation vs lifecycle control.
- Decision: Keep `Esc` as navigation-only (`Back`), and add `p` as explicit pause action for the current target (`PauseTarget`).
- Rationale: This directly matches the requested user intent and avoids implicit stop side effects.
- Alternative considered: `Esc` toggles pause. Rejected because it overloads navigation key semantics and causes accidental data loss.

4. Persist and expose per-target state (`Monitoring`, `Paused`, `Waiting`).
- Decision: Track monitor session runtime state separately from discovered-address data and show state in monitor header (and selector metadata when available).
- Rationale: Users need to understand whether a target is still collecting after switching views.
- Alternative considered: Keep state internal only. Rejected because behavior becomes opaque and hard to verify.

## Risks / Trade-offs

- [More concurrent monitor tasks can increase CPU/memory usage] -> Mitigation: cap polling work to existing cadence, cleanly terminate paused sessions, and avoid duplicate sessions per target.
- [State drift between background session and monitor view] -> Mitigation: centralize session snapshot updates in one owner and have views render from that source only.
- [Input ambiguity between `p` pause and existing shortcuts] -> Mitigation: reserve lowercase/uppercase `p` only for pause and keep all current shortcuts unchanged.
- [Shutdown complexity with many sessions] -> Mitigation: implement deterministic teardown that iterates all sessions, sends shutdown, awaits joins, then persists history once.

## Migration Plan

1. Refactor TUI main loop to create and keep a session registry across app selections.
2. Add session lifecycle operations: `ensure_running(target)`, `pause(target)`, `attach_view(target)`.
3. Extend monitor-view action enum and key handling to emit `PauseTarget` on `p`.
4. Update selector/monitor rendering to show runtime state and preserve per-target cached data boundaries.
5. Add/adjust tests for multi-target persistence, `Esc` behavior, and explicit `p` pause semantics.

Rollback: restore pre-change single-target lifecycle by removing session registry and reverting `Esc` teardown semantics.

## Open Questions

- Should selecting a paused target auto-resume monitoring immediately, or require an explicit resume key? This change assumes auto-resume on selection for minimal workflow friction.
