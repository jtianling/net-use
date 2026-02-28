## 1. Sorting State Model

- [x] 1.1 在 `src/tui/monitor_view.rs` 新增地址排序模式枚举（`Time`/`Alphabetical`）与标签/切换方法
- [x] 1.2 在 `MonitorView` 增加排序模式字段并初始化为 `Time`（发现顺序默认）
- [x] 1.3 为当前 IPv4/IPv6 条目选择逻辑增加“按排序模式返回渲染视图”的辅助函数（不破坏底层存储顺序）

## 2. Keyboard + Rendering Integration

- [x] 2.1 在监控页按键处理中新增 `o`/`O` 热键，切换排序模式并写入状态消息
- [x] 2.2 更新 IPv4/IPv6 列表渲染以使用排序后的视图（Masked/Raw 与 Time/Alphabetical 组合均生效）
- [x] 2.3 更新状态栏信息与快捷键提示文案，展示当前排序模式并包含 `[O]rder` 帮助

## 3. Verification

- [x] 3.1 新增/更新单元测试覆盖排序模式切换行为（默认 Time、切换到 Alphabetical、再次切回）
- [x] 3.2 新增/更新测试验证排序与显示模式联动（Masked/Raw 下 IPv4 与 IPv6 同步切换）
- [x] 3.3 运行 `cargo test` 并修复与本变更相关的失败，确保监控视图行为与规格一致
