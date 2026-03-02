## Why

现在一个 target 一旦被选中并进入监控，就会在后台持续监控，直到用户显式暂停。回到 app 选择列表时，用户无法直接看出哪些 app 已在监控中，容易重复选择并产生混淆。

## What Changes

- 在 app 选择列表中为每个条目显示“是否正在被监控”的状态标识。
- 已在监控中的 target 在列表中应有清晰且稳定的可视化状态（例如 `MONITORING` / `PAUSED`）。
- 未被监控的 target 在列表中显示默认未监控状态，帮助用户快速区分当前会话中的监控覆盖范围。

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: 扩展 app selection screen 的状态展示要求，使其反映每个可选 target 的当前监控状态。

## Impact

- Affected code: TUI selector row model, selector rendering, and session-state lookup used by app list entries.
- No CLI flag/API contract change.
- No new external dependency expected.
