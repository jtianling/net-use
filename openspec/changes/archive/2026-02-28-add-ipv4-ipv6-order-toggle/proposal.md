## Why

当前监控页的 IPv4/IPv6 地址列表只有固定排序方式，用户无法在“按发现时间（默认）”与“按字母表”之间切换，不利于不同场景下的比对与排查。增加统一的排序切换热键可以提升可读性与操作效率。

## What Changes

- 在监控页新增按键 `o`（order），用于切换地址列表排序模式。
- 排序模式覆盖 IPv4 与 IPv6 两个列表，并保持同一种全局模式。
- 默认排序保持为按发现时间（最先发现的顺序）；切换后支持按字母表排序。
- 在界面状态区显示当前排序模式，便于用户确认当前视图语义。

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: 监控页地址列表新增排序模式切换（时间顺序/字母表顺序）及对应键盘交互与状态展示。

## Impact

- Affected specs: `openspec/specs/tui/spec.md`（需求增量）
- Affected code: TUI 键盘事件处理、地址列表渲染与排序逻辑、状态栏/帮助文案
- Backward compatibility: 默认行为保持按时间排序，无破坏性变更
