## 1. Project Setup

- [x] 1.1 Initialize Rust project with `cargo init`, configure Cargo.toml with dependencies: clap, tokio, ratatui, crossterm, plist
- [x] 1.2 Create module structure: `monitor/`, `discovery/`, `tui/`, `types.rs`, `app.rs`
- [x] 1.3 Define CLI arguments with clap: `--pid`, `--name`, `--bundle`, `--no-tui`
- [x] 1.4 Add root permission check at startup with friendly error message

## 2. Core Types

- [x] 2.1 Define `AppInfo` struct (display name, bundle id, executable name, app path)
- [x] 2.2 Define `ProcessInfo` struct (pid, name, executable path)
- [x] 2.3 Define `MonitorTarget` enum (ByPid, ByName, ByBundle)
- [x] 2.4 Define `DiscoveredAddress` enum (Ipv4Subnet for `x.x.x.0/24` format, Ipv6Full for complete address)
- [x] 2.5 Define `MonitorEvent` enum for channel messages (NewAddress, ProcessAdded, ProcessRemoved, TargetLost, TargetFound)

## 3. macOS Process FFI

- [x] 3.1 Create safe Rust wrappers for `proc_listallpids` (enumerate all PIDs)
- [x] 3.2 Create safe Rust wrapper for `proc_pidpath` (get executable path for a PID)
- [x] 3.3 Create safe Rust wrapper for `proc_pidinfo` with `PROC_PIDLISTFDS` (list file descriptors)
- [x] 3.4 Create safe Rust wrapper for `proc_pidfdinfo` with `PROC_PIDFDSOCKETINFO` (get socket details)
- [x] 3.5 Create safe Rust wrapper for `proc_listchildpids` (list child PIDs)
- [x] 3.6 Extract remote IP address (IPv4 and IPv6) from `socket_fdinfo` struct

## 4. App Discovery

- [x] 4.1 Implement `/Applications` and `~/Applications` directory scanning for `.app` bundles
- [x] 4.2 Implement `Info.plist` parsing to extract CFBundleIdentifier, CFBundleName/CFBundleDisplayName, CFBundleExecutable
- [x] 4.3 Implement running process enumeration: list all PIDs, get paths, match against known app bundles
- [x] 4.4 Merge installed and running info into a unified app list with running status

## 5. Process Monitor

- [x] 5.1 Implement target resolution: PID lookup (verify exists), name matching (scan all procs), Bundle ID resolution (Info.plist → executable name → scan procs by path)
- [x] 5.2 Implement child process tree discovery using `proc_listchildpids` recursively
- [x] 5.3 Implement process tree refresh: detect new children, remove exited processes each polling cycle
- [x] 5.4 Implement waiting mode: when target not found, poll process list periodically until target appears
- [x] 5.5 Implement app restart detection: detect target loss, switch to waiting mode, resume on reappearance

## 6. Connection Collector

- [x] 6.1 Implement socket fd enumeration for a single PID: list fds, filter socket type, call proc_pidfdinfo
- [x] 6.2 Implement IPv4 /24 aggregation: `ip & 0xFFFFFF00`, format as `x.x.x.0/24`
- [x] 6.3 Implement IPv6 full address recording
- [x] 6.4 Implement address filtering: skip loopback (127.0.0.0/8, ::1), link-local (169.254.0.0/16, fe80::/10)
- [x] 6.5 Implement deduplication with HashSet, emit MonitorEvent::NewAddress only for new entries
- [x] 6.6 Implement the main monitor loop: tokio task, 200ms interval, process tree refresh + socket scan + emit events via mpsc channel

## 7. TUI - App Selection Screen

- [x] 7.1 Set up ratatui + crossterm terminal initialization and cleanup (raw mode, alternate screen)
- [x] 7.2 Implement app list rendering: running apps section (with PID), installed-only apps section
- [x] 7.3 Implement text filter input: capture keystrokes, filter app list by name/bundle ID case-insensitively
- [x] 7.4 Implement list navigation: arrow keys to move cursor, Enter to select
- [x] 7.5 Implement Q to quit from selection screen

## 8. TUI - Monitoring Screen

- [x] 8.1 Implement monitoring screen layout: target info header, tracked processes panel, IPv4 subnets panel, IPv6 addresses panel, status bar
- [x] 8.2 Wire up mpsc channel receiver: update UI state on MonitorEvent messages
- [x] 8.3 Implement stability indicator: track time since last new IP, display "No new IPs for X min" in status bar
- [x] 8.4 Implement Export (E key): write all addresses to a text file, one per line, show file path in status bar
- [x] 8.5 Implement Copy (C key): copy all addresses to macOS clipboard via `pbcopy`
- [x] 8.6 Implement Esc to return to app selection screen (stop monitoring, preserve data)
- [x] 8.7 Implement Q to quit from monitoring screen

## 9. CLI Output Mode

- [x] 9.1 Implement `--no-tui` mode: validate that a target is specified, skip TUI initialization
- [x] 9.2 Implement stdout output loop: receive MonitorEvent::NewAddress, println one line per new IP
- [x] 9.3 Implement SIGINT handling: clean exit on Ctrl+C without extra output

## 10. Integration and Polish

- [x] 10.1 Wire up main.rs: CLI parsing → mode selection → TUI app or CLI output loop
- [x] 10.2 Handle edge cases: permission denied errors on proc APIs, process disappearing mid-scan
- [x] 10.3 Test with real apps: GUI app (e.g., Chrome), CLI tool (e.g., curl), multi-process app
