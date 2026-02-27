## Context

当前实现只把运行中 PID 与已安装 `.app` 路径做前缀匹配, 因此 `discover_running_apps` 只能产出 GUI app 对应进程.  `AppSelector` 实际渲染的是这份结果, 所以用户在无参数启动时看不到独立的 CLI 进程.  同时监控页 `ProcessInfo` 仅有 `pid + name`, 无法区分同名 CLI 子进程.

约束:
- 运行环境是 macOS, 进程信息来源是 `libproc`.
- 现有 `MonitorTarget` 与事件流已经稳定, 变更应尽量兼容.
- 不引入新依赖, 复用当前 `process_tree` 模块.

## Goals / Non-Goals

**Goals:**
- 在应用选择页展示可选的运行中 CLI 进程.
- 对 GUI 与 CLI 进程使用统一 `AppInfo` 输出模型, 保持后续选择逻辑不变.
- 在监控页的进程列表增加命令行可视信息, 提高同名进程可辨识度.
- 保持现有参数模式(`--pid/--name/--bundle`)和 no-tui 行为不变.

**Non-Goals:**
- 不实现完整进程管理器视图, 不展示全部系统字段.
- 不修改网络连接采集逻辑.
- 不新增跨平台适配, 仍以 macOS 为目标.

## Decisions

1. 在 `running_apps` 中增加 CLI 进程补集流程.
- 决策: 先生成现有 GUI 运行集, 再遍历全部 PID, 对未命中 `.app` 路径且可读取名称的进程生成 CLI `AppInfo`.
- CLI `AppInfo` 约定: `bundle_id=None`, `app_path=None`, `executable_name=process_name`, `display_name=process_name`, `pid=Some(pid)`.
- 备选方案: 直接替换为全量 PID 枚举并动态反查 bundle.  放弃原因是会重写现有稳定路径且复杂度更高.

2. 更新去重键, 避免 bundle-only 去重误伤 CLI.
- 决策: 对运行态条目采用 `bundle_id` 优先, 为空时退化到 `pid` 维度去重.
- 备选方案: 仅按 `display_name` 去重.  放弃原因是同名不同进程会被合并, 丢失可选目标.

3. 扩展 `ProcessInfo` 增加 `command` 字段, 监控页优先展示命令行摘要.
- 决策: 在 `process_tree` 新增读取命令行函数, `get_process_info` 同步填充可选命令行字符串.
- UI 展示规则: 有命令行时显示 `PID + name + command_snippet`, 无命令行时回退 `PID + name`.
- 备选方案: 仅显示 PID 与 name.  放弃原因是不能解决用户反馈的 CLI 可见性问题.

4. `AppSelector` 增加来源标识与过滤兼容.
- 决策: 列表行增加轻量来源标签(`GUI` / `CLI`), 过滤仍按 `display_name + bundle_id` 并补充 `pid` 文本匹配.
- 备选方案: 增加多列复杂布局.  放弃原因是改动范围大, 与最小变更原则冲突.

## Risks / Trade-offs

- [Risk] CLI 进程数量大, 列表噪声变高.  -> Mitigation: 保留输入过滤, 并将运行中的 GUI 条目优先排序在前.
- [Risk] 某些 PID 命令行读取失败.  -> Mitigation: `command` 使用 `Option<String>`, UI 自动回退到 name.
- [Risk] 频繁进程创建导致列表抖动.  -> Mitigation: 选择页仍是快照模型, 不做实时刷新, 避免交互抖动.

## Migration Plan

- 变更属于纯运行时逻辑与显示层增强, 无持久化数据迁移.
- 发布步骤: 合并代码后按现有方式构建与运行.
- 回滚策略: 若出现异常, 可直接回滚本次变更提交, 不影响外部接口与数据.

## Open Questions

- 命令行摘要最大长度是否限制为固定宽度(例如 48 字符), 还是按终端宽度动态截断.
- CLI 进程是否需要额外过滤掉系统守护进程, 当前先按最小变更保留全量可见策略.
