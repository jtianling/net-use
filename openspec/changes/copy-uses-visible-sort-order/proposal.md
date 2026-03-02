## Why

监控页已经支持通过 `O` 在“发现顺序”和“排序后顺序”之间切换，但用户按 `C` 复制时仍总是按默认时间顺序输出，导致“屏幕所见”与“复制所得”不一致，容易误导后续粘贴和排查流程。

## What Changes

- 调整监控页复制逻辑：按 `C` 时复制内容必须与当前屏幕显示顺序一致，而不是固定按默认发现顺序。
- 复制时基于当前屏幕视图（`S` 显示模式 + `O` 排序模式）输出，确保内容类型与顺序都与当前界面一致，并保持每行一条的格式。
- 明确并覆盖测试：当用户先按 `O` 切换顺序再按 `C` 复制时，复制结果顺序与当前可见顺序一致。

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: 调整“Copy to clipboard”行为要求，使复制结果顺序与监控页当前显示顺序保持一致。

## Impact

- Affected code: monitoring screen key handling, clipboard export/copy path, and address list ordering helper usage in TUI state.
- No CLI/API contract change.
- No new external dependency expected.
