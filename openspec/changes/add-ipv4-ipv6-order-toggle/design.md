## Context

当前监控视图已有地址显示模式切换（`S`: Masked/Raw），IPv4 与 IPv6 数据分别以 `Vec<String>` 按发现先后追加保存，并通过 `HashSet` 去重。现状缺少排序模式切换，导致用户在长列表中难以按字母快速定位，也无法显式回到“按发现时间”语义。

本变更只涉及 TUI 交互与渲染层，不改变采集器、去重规则与导出/复制数据语义。

## Goals / Non-Goals

**Goals:**
- 新增 `o`/`O` 热键切换地址排序模式：`Time`（默认）与 `Alphabetical`。
- 排序模式同时作用于 IPv4 与 IPv6 当前显示列表（Masked/Raw 任一模式下均生效）。
- 默认行为保持与当前一致（按发现顺序）。
- 在状态栏和快捷键提示中展示排序模式与按键说明。

**Non-Goals:**
- 不改变地址采集、聚合、去重逻辑。
- 不新增持久化配置（本次不记忆用户上次排序偏好）。
- 不改变导出/复制内容顺序规则（仍沿用现有 canonical 输出）。

## Decisions

1. 引入独立排序状态 `AddressOrderMode`（`Time` / `Alphabetical`）
- Rationale: 与现有 `AddressDisplayMode`（Masked/Raw）正交，避免把两类状态耦合成笛卡尔组合分支，降低事件处理复杂度。
- Alternative considered: 复用或扩展 `AddressDisplayMode`。缺点是语义混杂，后续再加排序策略（例如 reverse）会导致枚举膨胀。

2. `Time` 模式直接复用现有 `Vec` 的插入顺序
- Rationale: 现有数据结构已天然保留发现顺序，无需为每条地址新增时间戳字段，改动最小且与当前行为一致。
- Alternative considered: 为地址记录显式 timestamp 再排序。缺点是引入额外模型复杂度，当前需求不需要。

3. `Alphabetical` 模式在渲染时生成排序视图（克隆并排序）
- Rationale: 不改动底层存储顺序，切回 `Time` 时可无损恢复；实现风险低。
- Alternative considered: 每次切换时原地排序并维护两份缓存。缺点是状态同步复杂，容易在新数据到达时出现顺序不一致。

4. 键位与反馈
- Decision: 监听 `o`/`O` 触发排序切换；状态栏增加 `Order: Time|A-Z`，并在短暂状态消息中提示切换结果。
- Alternative considered: 只在列表标题展示排序模式。缺点是可见性不足，用户不易发现当前全局状态。

## Risks / Trade-offs

- [Risk] 渲染时排序在大列表下增加 CPU 开销  
  → Mitigation: 当前 TUI 刷新频率和地址量规模可接受；排序仅对当前可见数据执行，且字符串比较成本可控。

- [Risk] 用户误解“时间排序”为最近优先而非发现先后  
  → Mitigation: 在文案中明确为 `Order: Time (discovery)` 或等价表达，避免歧义。

- [Risk] 新增热键与现有热键提示不一致  
  → Mitigation: 同步更新状态栏快捷键帮助文本并补充单元测试覆盖按键行为。

## Migration Plan

- 本变更为纯 TUI 行为增强，无数据迁移。
- 通过常规发布即可生效；若需回滚，移除排序模式状态与 `o` 热键处理即可，风险可控。

## Open Questions

- 排序比较是否需要 locale-aware/自然排序（例如数字段按数值比较）？当前默认采用标准字符串字典序。
- 状态栏文案使用 `Alphabetical` 还是 `A-Z`；实现时以可读性优先并保持统一。
