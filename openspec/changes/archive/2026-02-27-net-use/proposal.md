## Why

macOS 下缺少一个简单易用的工具来查看某个特定 app 实际访问了哪些 IP 地址.  构建防火墙白名单时, 需要先运行目标 app 一段时间, 收集其所有网络连接的目标 IP, 再聚合为子网列表.  现有工具 (lsof, nettop) 要么不够直观, 要么无法持续追踪并自动去重.

## What Changes

- 新建 Rust CLI/TUI 工具 `net-use`, 支持以下核心能力:
  - 通过 PID, 进程名或 Bundle ID 指定监控目标, 也可通过 TUI 交互选择
  - 扫描 `/Applications` 发现已安装 app, 支持选择尚未启动的 app 并等待其运行
  - 使用 `proc_pidfdinfo` 系统调用轮询目标进程及其整个子进程树的 socket 连接
  - 采集远端 IP 地址, IPv4 聚合为 /24 子网 (最后一段归零), IPv6 保留完整地址
  - TUI 模式: 实时展示已发现的 IP 列表, 支持导出和复制
  - CLI 模式 (`--no-tui`): 发现新 IP 时逐行输出到 stdout, 管道友好
- 需要 sudo 权限运行 (读取其他进程的 socket fd)
- 仅支持 macOS

## Capabilities

### New Capabilities
- `process-monitor`: 进程发现与追踪 — 通过 PID/进程名/Bundle ID 定位目标进程, 递归追踪整个子进程树, 处理子进程动态增减和 app 重启
- `connection-collector`: 网络连接采集 — 通过 proc_pidfdinfo 轮询 socket fd, 提取远端 IP 地址, IPv4 聚合为 /24 子网, IPv6 保留完整地址, 去重存储
- `app-discovery`: 应用发现 — 扫描 /Applications 和 ~/Applications, 解析 Info.plist 获取 Bundle ID 和可执行文件名, 列出已安装和运行中的 app
- `tui`: 终端交互界面 — app 选择界面 (筛选, 分组显示), 监控主界面 (进程列表, IP 列表, 稳定性提示), 导出和复制操作
- `cli-output`: 命令行输出模式 — 无 TUI 的纯文本输出, 发现新 IP 即打印一行, 适合管道和脚本使用

### Modified Capabilities

(无, 全新项目)

## Impact

- 新项目, 无现有代码受影响
- 依赖 macOS 专有系统调用 (`proc_pidfdinfo`, `proc_listchildpids`), 不可移植到其他平台
- 主要外部依赖: `clap` (CLI 参数), `ratatui` + `crossterm` (TUI), `plist` (Info.plist 解析)
- 需要 root 权限运行
