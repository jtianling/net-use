## 1. Session Registry Refactor

- [x] 1.1 Refactor `tui_main_loop` in `src/main.rs` to maintain a long-lived per-target session registry instead of creating exactly one monitor loop per screen visit.
- [x] 1.2 Add session lifecycle helpers (`ensure_running`, `pause_target`, `shutdown_all`) that manage monitor task handles and shutdown channels per `MonitorTarget`.
- [x] 1.3 Ensure session snapshots keep per-target discovered address history updated while the target is monitored in background.

## 2. Monitor View Lifecycle Controls

- [x] 2.1 Extend `MonitorAction` in `src/tui/monitor_view.rs` with an explicit pause action and bind `p`/`P` to emit it.
- [x] 2.2 Keep `Esc` behavior navigation-only so leaving a monitor view does not stop the target session.
- [x] 2.3 Update monitor header/status messaging to distinguish active, waiting, and paused target states without clearing collected address lists.

## 3. Selector And Target Re-entry Behavior

- [x] 3.1 Update selection flow so selecting an already monitored target re-attaches to its existing session state instead of resetting data.
- [x] 3.2 Ensure selecting another target starts or resumes that target while previously selected targets continue monitoring unless explicitly paused.
- [x] 3.3 Preserve strict per-target data isolation when switching back and forth between targets.

## 4. Persistence, Teardown, And Verification

- [x] 4.1 Update quit-path teardown to stop all active sessions and persist the full per-target history map once before exit.
- [x] 4.2 Add/adjust tests in `src/main.rs` and `src/tui/monitor_view.rs` for `Esc` non-stop behavior, `p` pause semantics, and multi-target isolation.
- [ ] 4.3 Manually validate TUI workflow: monitor A -> Esc -> monitor B -> confirm A still collecting -> reopen A -> press `p` -> confirm A stops while B continues.
