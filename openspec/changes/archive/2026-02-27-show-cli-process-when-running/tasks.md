## 1. 运行进程发现补齐

- [x] 1.1 在 `src/discovery/running_apps.rs` 增加 CLI 进程补集逻辑, 让无 bundle 的运行中进程也生成 `AppInfo`
- [x] 1.2 调整运行结果的去重与排序规则, 保证 GUI 条目优先且 CLI 条目按 PID 可区分

## 2. 进程元数据扩展

- [x] 2.1 在 `src/types.rs` 扩展 `ProcessInfo` 结构, 增加可选命令行字段并保持事件链路兼容
- [x] 2.2 在 `src/monitor/process_tree.rs` 实现命令行读取能力并接入 `get_process_info`, 失败时安全回退

## 3. TUI 展示更新

- [x] 3.1 在 `src/tui/app_selector.rs` 增加 GUI/CLI 来源标识, 并让过滤支持 PID 文本匹配
- [x] 3.2 在 `src/tui/monitor_view.rs` 更新进程列表渲染, 展示 `PID + name + command summary` 且提供无命令行回退

## 4. 测试与回归验证

- [x] 4.1 为运行进程发现与去重规则补充单元测试, 覆盖 CLI 可见与同名多 PID 场景
- [x] 4.2 为监控视图进程展示补充单元测试, 覆盖命令行可用与不可用两种路径
- [x] 4.3 执行 `cargo test` 验证改动不破坏现有行为
