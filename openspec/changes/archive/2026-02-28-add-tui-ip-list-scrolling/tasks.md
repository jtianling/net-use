## 1. Scroll State Foundations

- [x] 1.1 Add address viewport state (`scroll_offset`, helper clamp/max methods) to `MonitorView` in `src/tui/monitor_view.rs`.
- [x] 1.2 Implement utilities to flatten current IPv4/IPv6 display rows into a single ordered address view for windowed rendering.
- [x] 1.3 Recompute and clamp viewport bounds whenever address counts or terminal layout-derived visible rows change.

## 2. Input Handling For Scrolling

- [x] 2.1 Add monitor-view key handling for line scroll up/down (`Up`/`k`, `Down`/`j`).
- [x] 2.2 Implement dual-pane scroll keys: `Up`/`Down` for IPv4, `j`/`k` for IPv6, one line at a time.
- [x] 2.3 Ensure scroll keys are bounded and do not regress existing shortcuts (`S`, `E`, `C`, `Esc`, `Q`).

## 3. Monitoring Render Refactor

- [x] 3.1 Refactor monitoring layout so header/process/status stay fixed while address content is rendered through a scroll window.
- [x] 3.2 Update IPv4/IPv6 address rendering to display only visible rows from current offset while preserving section labels and counts.
- [x] 3.3 Add overflow indicator text (for example `Lines X-Y / N` with up/down hints) when total rows exceed viewport capacity.

## 4. Verification

- [x] 4.1 Add/extend tests for scroll offset clamping and key-driven offset transitions in `src/tui/monitor_view.rs` tests.
- [x] 4.2 Add a regression test confirming address display mode toggle (`S`) still affects both IPv4/IPv6 views while scrolled.
- [x] 4.3 Manually validate with high-volume IPv4/IPv6 data and terminal resize to confirm stable bounded scrolling behavior.
