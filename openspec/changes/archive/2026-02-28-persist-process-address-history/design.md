## Context

当前 TUI 主循环 (`tui_main_loop`) 只维护一个 `preserved: Option<(MonitorTarget, ...)>` 快照。 这意味着仅“上一次监控目标”的地址能被恢复：若用户依次选择 A -> B -> A，第二次进入 A 会丢失第一次采集的历史地址。 现有 `MonitorView` 已支持 `restore_data`，但缺少“按目标长期缓存”的外层状态容器。

## Goals / Non-Goals

**Goals:**
- 在同一次 `net-use` 运行期间，按监控目标保存地址历史（IPv4 masked/raw 与 IPv6 masked/raw）。
- 用户从监控页返回选择页后，再次选择同一目标时，监控页立即恢复该目标历史地址。
- 保持现有采集、导出、复制、地址显示模式等行为不变。

**Non-Goals:**
- 不实现跨程序重启的磁盘持久化。
- 不改变“目标身份”语义（PID 目标仍按 PID 匹配，Bundle 目标仍按 Bundle ID 匹配）。
- 不新增历史管理 UI（例如清理按钮、历史列表页面）。

## Decisions

### 1. 将单一 `preserved` 快照改为按目标索引的缓存表

**选择**: 在 `tui_main_loop` 中使用 `HashMap<MonitorTarget, PreservedData>` 保存历史，其中 `PreservedData` 封装四类地址列表（IPv4 masked/raw、IPv6 masked/raw）。

**备选方案**:
- 继续使用单一 `Option`，仅保存最近一次目标。
- 引入全局静态缓存或额外状态管理模块。

**理由**: `HashMap` 方案对现有结构侵入最小，且能直接覆盖 A -> B -> A 场景，不需要改变 `MonitorView` 接口。

### 2. 以 `MonitorTarget` 作为缓存键

**选择**: 为 `MonitorTarget` 补充 `Eq` 和 `Hash` 派生，实现 `HashMap` 键能力；键匹配规则与现有目标语义一致。

**备选方案**:
- 转换为字符串键（如 `"pid:123"`, `"bundle:com.foo"`）。
- 使用 `AppInfo` 作为键。

**理由**: 直接使用 `MonitorTarget` 可复用已有目标类型，避免新增编码/解码规则和潜在冲突。

### 3. 历史写入时机保持“退出监控页时”

**选择**: 保持当前时机，在 `view.run(...)` 返回后写回缓存（`insert(target.clone(), snapshot)`）。

**备选方案**:
- 在事件循环中增量同步缓存。
- 周期性定时写回。

**理由**: 退出时写回足以满足“再次选中恢复”需求，实现简单且无额外并发复杂度。

### 4. 恢复策略为“命中则恢复，不命中则空状态”

**选择**: 新建 `MonitorView` 后，若缓存存在 `target` 对应快照则调用 `restore_data`；否则保持当前空初始状态。

**备选方案**:
- 对未命中目标复用最近一次快照。
- 对同名不同 PID 进行模糊合并。

**理由**: 严格按目标命中可避免错误复用地址，行为可预测。

## Risks / Trade-offs

- [会话内存增长] -> 监控过多个目标时缓存条目增加。缓解: 当前仅保存去重后的地址字符串，体量可控；后续可再加 LRU 上限。
- [PID 目标天然不稳定] -> 进程重启导致 PID 变化时无法命中历史。缓解: 该行为与现有 PID 语义一致，规范中明确“标识变化按新目标处理”。
- [缓存数据可能陈旧] -> 恢复后展示历史+新采集混合数据。缓解: 这符合“保留历史记录”目标，不做自动过期。

## Migration Plan

1. 在类型层为 `MonitorTarget` 增加 `Eq`、`Hash` 派生。
2. 在 `tui_main_loop` 引入 `HashMap<MonitorTarget, PreservedData>` 替换单一 `preserved`。
3. 将“进入监控页恢复”和“离开监控页写回”逻辑切换为按目标键读写。
4. 补充单元测试，覆盖 A -> B -> A 重新进入时恢复地址历史的场景。

回滚策略: 若实现引入问题，可回退到单一 `preserved` 快照逻辑（仅保留最近一次目标）。

## Open Questions

- 历史缓存是否需要会话内上限（目标数量或地址条数）以防极端长时间运行内存膨胀。
