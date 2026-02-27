## Why

当前 TUI 运行时列表主要来源于已安装 .app 的匹配结果, 导致大量正在运行的命令行进程不可见.  这让用户无法直接选择和监控 CLI 程序, 与工具的实际使用场景不一致.

## What Changes

- 在应用发现阶段补充 CLI 进程枚举路径, 让未对应 .app bundle 的运行中进程也能进入候选列表.
- 调整应用列表合并与去重规则, 避免仅按 bundle 维度去重导致 CLI 进程被错误折叠.
- 在监控界面的进程区域展示更完整的进程标识, 包括可用的命令行信息, 便于区分 GUI 进程与 CLI 进程.
- 明确 CLI 进程的显示优先级与命名策略, 保证列表可读性和稳定性.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `app-discovery`: 修改运行中进程发现规则, 明确必须包含无 bundle 的 CLI 进程并提供可选择元数据.
- `tui`: 修改选择页与监控页的展示要求, 明确 CLI 进程需要可见且可区分.

## Impact

- Affected code:
  - `src/discovery/running_apps.rs`
  - `src/tui/app_selector.rs`
  - `src/tui/monitor_view.rs`
  - `src/types.rs`
  - `src/monitor/process_tree.rs`
- API/CLI: 无外部参数变更, 仅行为与显示增强.
- Dependencies: 无新增三方依赖, 优先复用现有 macOS 进程查询能力.
