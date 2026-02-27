## Context

全新 Rust 项目, 目标是在 macOS 上监控指定 app 的网络连接, 输出去重后的 IP/子网列表用于构建防火墙白名单.  macOS 提供 `libproc` 系列系统调用 (`proc_pidinfo`, `proc_pidfdinfo`, `proc_listchildpids`) 可以读取任意进程的 socket 信息, 需要 root 权限.

## Goals / Non-Goals

**Goals:**
- 准确采集目标 app (含完整子进程树) 的所有 TCP/UDP 远端 IP
- IPv4 聚合为 /24 子网 (最后一段归零), IPv6 保留完整地址
- 提供 TUI 交互模式和无 TUI 的纯文本 stdout 模式
- 支持监控尚未启动的 app, 检测到启动后自动开始采集
- app 退出后保留数据, 重新出现时累加

**Non-Goals:**
- 不支持 macOS 以外的平台
- 不做流量抓包或深度包检测 (DPI)
- 不做 DNS 解析 (只记录 IP, 不反查域名)
- 不做持久化存储 (进程退出即丢弃, 通过导出保存)
- 不做实时流量统计 (只关心去重后的 IP 集合)

## Decisions

### 1. 监控方式: `proc_pidfdinfo` 轮询

**选择**: 以 ~200ms 间隔轮询 `proc_pidfdinfo`, 遍历目标进程的所有 socket fd, 提取远端地址.

**备选方案**:
- dtrace hook connect() — macOS 开启 SIP 后基本不可用
- Network Extension — 需要 Apple 开发者证书 + 系统扩展签名, 过重
- libpcap 抓包 + 进程关联 — 抓包容易, 但将包关联到具体进程在 macOS 上非常困难
- 轮询 lsof -i — 本质相同但通过子进程解析文本, 效率低且不可靠

**理由**: `proc_pidfdinfo` 是 macOS 上唯一不需要特殊签名/SIP 豁免的 per-process socket 监控方式.  轮询间隔 200ms 可以捕获绝大多数连接, 极短生命周期的连接 (<200ms) 可能漏掉, 但对白名单场景影响不大 — 长期运行总会捕获到.

### 2. 进程树追踪策略

**选择**: 每轮轮询时, 用 `proc_listchildpids` 递归获取目标主进程的所有后代进程, 动态更新追踪集合.

**关键细节**:
- 通过 Bundle ID 或进程名定位时, 先遍历系统进程列表 (`proc_listallpids`) 找到匹配的主进程
- 通过 Bundle ID 定位时, 从 app bundle 的 `Info.plist` 读取 `CFBundleExecutable`, 匹配进程的可执行文件路径 (`proc_pidpath`)
- 监控未运行的 app 时, 持续轮询进程列表等待匹配进程出现
- 主进程退出后, 保留已采集数据, 回到等待状态

### 3. IP 聚合规则

**选择**:
- IPv4: `ip & 0xFFFFFF00` 后格式化为 `x.x.x.0/24`
- IPv6: 保留完整地址, 不做掩码
- 用 `HashSet<String>` 去重, key 为格式化后的字符串

**理由**: 用户明确要求 IPv4 /24 掩码, IPv6 完整地址.  字符串作为 key 简单直接, 性能在此场景下不是瓶颈 (去重集合不会超过几千条).

### 4. TUI 框架: ratatui + crossterm

**选择**: `ratatui` 渲染 + `crossterm` 处理终端事件.

**备选方案**:
- Tauri (Rust + React) — 功能强大但对 "需要 sudo" 的工具来说过重
- egui — 即时模式 GUI, 外观不够原生, 对此场景无明显优势

**理由**: ratatui 是 Rust TUI 生态的主流选择, 轻量, 适合这类 "实时监控 + 列表展示" 的场景.  与 sudo 运行方式天然兼容.

### 5. 应用架构

**选择**: 单二进制, 异步运行时 (tokio), 事件驱动.

```
main
 ├── CLI 参数解析 (clap)
 ├── 如果有 --pid/--name/--bundle 且 --no-tui:
 │     直接进入 CLI 输出模式
 └── 否则:
       进入 TUI 模式

监控循环 (独立 tokio task):
  loop {
      sleep(200ms)
      更新进程树 (找新子进程, 移除已退出的)
      遍历所有追踪进程的 socket fd
      提取新 IP → 发送到 UI channel
  }

TUI 循环 (主线程):
  loop {
      处理键盘事件
      接收新 IP 通知 (从 channel)
      渲染界面
  }

CLI 输出循环:
  loop {
      接收新 IP 通知 (从 channel)
      println!("{ip}")
  }
```

监控引擎和 UI 通过 `tokio::sync::mpsc` channel 解耦, 两种输出模式共享同一个监控引擎.

### 6. TUI 界面分两个 screen

- **App 选择 screen**: 列出已安装 + 运行中的 app, 支持文本筛选, 回车选择后进入监控
- **监控 screen**: 显示追踪的进程列表, 已发现的 IPv4/IPv6 列表, 底部状态栏 (总数, 距上次新 IP 的时间, 快捷键)

两个 screen 之间可以切回 (按 Esc 回到选择).

## Risks / Trade-offs

- **短连接遗漏** → 200ms 轮询间隔下, 生命周期极短的连接可能漏掉.  缓解: 长期监控 + 白名单场景下, 重要连接会反复出现.  可考虑后续降低到 100ms.
- **macOS 版本兼容性** → `proc_pidfdinfo` 等 API 在不同 macOS 版本上行为可能有差异.  缓解: 开发时在当前系统验证, 文档标注最低支持版本.
- **root 权限** → 必须 sudo 运行, 对用户有一定门槛.  缓解: 启动时检测权限不足并给出友好提示.
- **多用户进程** → 某些 app 的 helper 进程可能不是主进程的直接子进程 (如通过 launchd 启动的 XPC service).  缓解: 初期只追踪进程树, 后续可扩展为通过可执行文件路径前缀匹配同 bundle 的进程.
