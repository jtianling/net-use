# net-use

[中文](README.zh-CN.md)

A per-app network connection monitor for macOS. Uses `proc_pidfdinfo` to track all TCP/UDP remote IPs of a target app and its entire process tree in real time, outputting a deduplicated address list — ideal for building firewall whitelists.

## Features

- Monitor the full process tree of a target app (including all child processes)
- Aggregate IPv4 addresses into /24 subnets; keep full IPv6 addresses
- Automatic deduplication — only newly discovered addresses are reported
- TUI interactive mode: browse installed apps, filter, and view live monitoring results
- CLI mode: plain text output, pipe-friendly for files or other commands
- Monitor apps that haven't started yet — collection begins automatically on launch
- Preserve data after app exit, accumulate on restart
- One-key export to file (`E`) or copy to clipboard (`C`)
- Pause/resume monitoring per target (`P`)
- Toggle between masked (subnet) and raw IP display (`S`)
- Toggle sort order: discovery time or alphabetical (`O`)
- Persist discovered addresses across sessions (default `/tmp/.net-use-address-history.json`, configurable via `--data-file`)

## Installation

Requires Rust 1.85+ (edition 2024).

```bash
cargo install net-use
```

Or build from source:

```bash
cargo build --release
```

The binary will be at `target/release/net-use`.

## Usage

Root privileges are required to read process socket information.

### TUI Mode

```bash
sudo net-use
```

Launches an app selection screen where you can type to filter, then press Enter to start monitoring.

Keybindings:

- `J` / `K` — Scroll address list down / up
- `S` — Toggle between masked (subnet) and raw IP view
- `O` — Toggle sort order (discovery time / alphabetical)
- `P` — Pause / resume monitoring
- `E` — Export address list to file
- `C` — Copy address list to clipboard
- `Esc` — Return to app selection
- `Q` — Quit

### CLI Mode

```bash
# Monitor by Bundle ID
sudo net-use --bundle com.google.Chrome --no-tui

# Monitor by process name
sudo net-use --name curl --no-tui

# Monitor by PID
sudo net-use --pid 1234 --no-tui

# Save history to a custom file
sudo net-use --data-file ./my-history.json

# Output to file
sudo net-use --bundle com.google.Chrome --no-tui > chrome-ips.txt
```

Example output:

```
142.250.80.0/24
172.217.14.0/24
2607:f8b0:4004:800::200e
```

## How It Works

1. Locate the target process via `proc_listallpids` and `proc_pidpath`
2. Recursively discover the full child process tree via `proc_listchildpids`
3. Poll every 200ms, enumerating all socket file descriptors with `proc_pidfdinfo(PROC_PIDFDSOCKETINFO)`
4. Extract remote IP addresses, filter out loopback and link-local, deduplicate, and output

## Limitations

- macOS only
- Requires root privileges (`sudo`)
- Very short-lived connections may be missed at the 200ms polling interval
- XPC services launched via launchd may not appear in the process tree

## License

MIT
