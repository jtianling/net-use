## 1. Target History Model

- [x] 1.1 Add `Eq` and `Hash` derives to `MonitorTarget` so it can be used as a cache key.
- [x] 1.2 Introduce a `PreservedData` structure in TUI flow to hold IPv4/IPv6 masked and raw address snapshots per target.

## 2. TUI History Persistence Flow

- [x] 2.1 Replace the single `preserved` snapshot in `tui_main_loop` with a `HashMap<MonitorTarget, PreservedData>` cache.
- [x] 2.2 Restore cached snapshot data into `MonitorView` when entering monitoring for a target with existing history.
- [x] 2.3 Save current `MonitorView` snapshot back into the per-target cache when leaving monitoring (Back/Quit paths).
- [x] 2.4 Ensure switching to another target never reuses a different target's cached addresses.

## 3. Verification

- [x] 3.1 Add/adjust unit tests for TUI flow to cover A -> B -> A re-selection with address restoration.
- [x] 3.2 Add/adjust tests for cache miss and target isolation (no cross-target address leakage).
- [x] 3.3 Run the project test suite and confirm all tests pass.
