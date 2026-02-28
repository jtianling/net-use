## Context

The monitoring TUI currently renders discovered IPv4 and IPv6 entries in a static vertical list. When the combined list grows beyond terminal height, entries below the fold become inaccessible. The current input model already handles action keys (export, copy, toggle mode, quit), so scroll behavior must integrate without breaking existing shortcuts or screen transitions.

## Goals / Non-Goals

**Goals:**
- Allow users to view all IPv4/IPv6 entries when the list exceeds the visible monitoring panel.
- Keep scroll behavior deterministic with clear bounds and visible progress feedback.
- Preserve existing monitoring interactions (`S`, `E`, `C`, `Esc`, `Q`) and render stability.
- Keep implementation localized to TUI state, input handling, and monitoring render logic.

**Non-Goals:**
- No mouse-wheel support in this change.
- No virtualization or pagination across multiple screens; this is in-panel vertical scrolling only.
- No changes to collector logic, export format, or clipboard payload ordering.

## Decisions

1. Introduce an address-list viewport state in monitoring mode.
- Decision: Track `scroll_offset` and `visible_rows` for the address list region.
- Rationale: Keeps scrolling independent from data collection and avoids mutating the underlying IP data.
- Alternative considered: Cursor-based selection per row. Rejected because requirement is read-only browsing and cursor UX adds complexity.

2. Define fixed key bindings for line/page movement.
- Decision: Support `Up`/`k` for one-line up, `Down`/`j` for one-line down, `PageUp` for page up, and `PageDown`/Space for page down.
- Rationale: Works in most terminal setups and aligns with common TUI conventions.
- Alternative considered: Reusing `Tab`/`Shift+Tab`. Rejected because those keys are often reserved by terminal/input handling and are less discoverable for scrolling.

3. Clamp scrolling and auto-adjust on data/resize changes.
- Decision: Clamp offset to `[0, max_offset]` each render tick, recomputing `max_offset = total_rows - visible_rows` after terminal resize or list size changes.
- Rationale: Prevents out-of-range access and stale offsets when content shrinks or viewport grows.
- Alternative considered: Only clamping on key events. Rejected because background updates can invalidate offsets between key events.

4. Render scoped scroll indicators in the monitoring panel.
- Decision: Show an indicator such as `Lines X-Y / N` plus up/down glyph hints when overflow exists.
- Rationale: Gives immediate feedback that more content exists and where the user is within the list.
- Alternative considered: No indicator. Rejected because hidden overflow would remain ambiguous.

5. Keep header/status/footer static and scroll only the address list block.
- Decision: Split monitoring layout into fixed sections and one scrollable content region.
- Rationale: Preserves situational awareness while browsing large lists.
- Alternative considered: Full-screen scroll of all monitoring content. Rejected because users would lose key status context.

## Risks / Trade-offs

- [Terminal key translation inconsistency] -> Mitigation: map multiple equivalent keys (`arrows` + `j/k`, `PageDown` + `Space`) and keep behavior additive.
- [Scroll math off-by-one errors] -> Mitigation: centralize clamp logic and add unit tests for edge cases (`0`, `visible_rows == total_rows`, dynamic shrink).
- [Frequent collector updates causing jumpy viewport] -> Mitigation: keep current offset stable unless it exceeds new bounds; do not auto-follow bottom in this change.
- [Small terminal heights reduce usable rows] -> Mitigation: ensure minimum visible_rows floor and degrade gracefully with truncated sections.

## Migration Plan

1. Add viewport state fields and helper methods in monitoring TUI state.
2. Extend key event handler with scroll actions.
3. Refactor monitoring renderer to compute visible window and indicators.
4. Add tests for clamp behavior and key-driven offset changes.
5. Validate manually with high-cardinality IPv4/IPv6 datasets and terminal resize.

Rollback: revert to previous renderer/input paths by removing viewport state usage; data model remains unchanged.

## Open Questions

- Should `Home`/`End` jump to top/bottom be included now or deferred?
- Should newly discovered addresses auto-scroll when user is currently at bottom?
