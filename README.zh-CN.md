# net-use

[English](README.md)

macOS 上的应用网络连接监控工具. 通过 `proc_pidfdinfo` 系统调用实时追踪指定 app 及其子进程树的所有 TCP/UDP 远端 IP, 输出去重后的地址列表, 适用于构建防火墙白名单.

## 功能

- 监控指定 app 的完整进程树 (含所有子进程)
- IPv4 地址聚合为 /24 子网, IPv6 保留完整地址
- 自动去重, 仅输出新发现的地址
- TUI 交互模式: 浏览已安装应用, 筛选, 实时查看监控结果
- CLI 模式: 纯文本输出, 可管道到文件或其他命令
- 支持监控尚未启动的 app, 检测到启动后自动开始采集
- app 退出后保留数据, 重新出现时累加
- 一键导出到文件 (`E`) 或复制到剪贴板 (`C`)
- 暂停/恢复监控 (`P`)
- 切换子网聚合和原始 IP 显示 (`S`)
- 切换排序方式: 发现顺序或字母序 (`O`)
- 跨会话持久化已发现的地址 (默认 `/tmp`, 可通过 `--data-dir` 自定义)

## 安装

需要 Rust 1.85+ (edition 2024).

```bash
cargo install net-use
```

或从源码构建:

```bash
cargo build --release
```

编译产物在 `target/release/net-use`.

## 使用

需要 root 权限才能读取进程 socket 信息.

### TUI 模式

```bash
sudo net-use
```

启动后进入应用选择界面, 支持输入文字筛选, 回车选择后进入监控界面.

快捷键:

- `J` / `K` — 向下/向上滚动地址列表
- `S` — 切换子网聚合和原始 IP 显示
- `O` — 切换排序方式 (发现顺序/字母序)
- `P` — 暂停/恢复监控
- `E` — 导出地址列表到文件
- `C` — 复制地址列表到剪贴板
- `Esc` — 返回应用选择
- `Q` — 退出

### CLI 模式

```bash
# 按 Bundle ID 监控
sudo net-use --bundle com.google.Chrome --no-tui

# 按进程名监控
sudo net-use --name curl --no-tui

# 按 PID 监控
sudo net-use --pid 1234 --no-tui

# 将历史数据保存到自定义目录
sudo net-use --data-dir ./my-data

# 输出到文件
sudo net-use --bundle com.google.Chrome --no-tui > chrome-ips.txt
```

输出示例:

```
142.250.80.0/24
172.217.14.0/24
2607:f8b0:4004:800::200e
```

## 工作原理

1. 通过 `proc_listallpids` 和 `proc_pidpath` 定位目标进程
2. 通过 `proc_listchildpids` 递归发现完整子进程树
3. 每 200ms 轮询一次, 通过 `proc_pidfdinfo(PROC_PIDFDSOCKETINFO)` 枚举所有 socket fd
4. 提取远端 IP 地址, 过滤 loopback 和 link-local, 去重后输出

## 限制

- 仅支持 macOS
- 需要 root 权限 (`sudo`)
- 200ms 轮询间隔下, 极短生命周期的连接可能漏掉
- 通过 launchd 启动的 XPC service 可能不在进程树中

## License

MIT
