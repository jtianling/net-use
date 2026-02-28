# TUI

## Purpose

Provide an interactive terminal user interface for app selection, real-time monitoring of network connections, and export of discovered IP addresses.

## Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing discovered apps and running CLI processes, with text filtering support, and SHALL restore persisted address history from the command working directory when a previously monitored target is selected again. The selector SHALL allow re-entering an already-active target monitor session without discarding in-memory collected data for that target.

#### Scenario: User opens the tool without --pid/--name/--bundle
- **WHEN** user runs `sudo net-use` without target arguments
- **THEN** system displays the app selection screen with running GUI apps and running CLI processes listed first, then installed-only apps

#### Scenario: User types a filter string
- **WHEN** user types "chr" in the filter input
- **THEN** the list narrows to entries whose display name, Bundle ID, or PID text contains "chr" case-insensitively

#### Scenario: User selects an app
- **WHEN** user highlights an app and presses Enter
- **THEN** system transitions to the monitoring screen and begins monitoring the selected app if it is not already being monitored in this session

#### Scenario: User selects a running CLI process
- **WHEN** user highlights a CLI process entry and presses Enter
- **THEN** system starts monitoring that PID as the target and shows it in the monitoring screen header

#### Scenario: User selects a target already monitored in this session
- **WHEN** user highlights a target that is already being monitored in the same `net-use` session and presses Enter
- **THEN** system opens that target's monitoring screen using the existing session state and continues collecting without resetting discovered address history

#### Scenario: User reselects a previously monitored target after restart
- **WHEN** user starts `net-use` in a working directory that contains persisted history and selects a target with saved records
- **THEN** system restores that target's previously discovered IPv4/IPv6 masked and raw address lists before appending newly collected results

#### Scenario: Persisted history file is missing or invalid
- **WHEN** system cannot read or parse the persisted history file in the working directory
- **THEN** system continues to app selection and monitoring with empty history without crashing

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes with PID, process name, and command summary when available, and discovered IPv4 and IPv6 addresses in switchable display and ordering modes. When the address list exceeds the monitoring viewport, the system SHALL provide bounded vertical scrolling for the address region while keeping header, status, and footer context visible. The monitoring screen SHALL support per-target pause/resume toggle via `p` and SHALL keep collected addresses visible after pause.

#### Scenario: Monitoring an active app
- **WHEN** monitoring is active
- **THEN** screen shows: target app name and Bundle ID, monitoring status and uptime, list of tracked PIDs with process names and command summary when present, list of discovered IPv4 subnets (/24), list of discovered IPv6 subnets (/64), with masked mode enabled by default and order mode set to discovery-time order by default

#### Scenario: New IP subnet discovered
- **WHEN** a new IPv4 subnet or IPv6 `/64` subnet is discovered
- **THEN** it appears in the list immediately on the next render cycle according to the current order mode

#### Scenario: User toggles address display mode
- **WHEN** user presses `S` on the monitoring screen
- **THEN** system toggles both IPv4 and IPv6 lists between masked and raw full-address display without losing collected data

#### Scenario: User toggles address order mode
- **WHEN** user presses `O` on the monitoring screen
- **THEN** system toggles address lists between discovery-time order and deterministic sorted order, where IPv4 entries are sorted by octet numeric value (for example `9.0.0.0` before `100.0.0.0`) and IPv6 entries are sorted alphabetically, without losing collected data

#### Scenario: User pauses current target monitoring
- **WHEN** user presses `p` or `P` on the monitoring screen for target A
- **THEN** system stops target A's active monitoring loop, marks target A as paused, and keeps target A's discovered IPv4/IPv6 masked and raw address history visible in the current view

#### Scenario: User resumes a paused target
- **WHEN** target A is paused and user presses `p` or `P` again
- **THEN** system restarts target A's monitoring loop, marks target A as active, and continues appending newly discovered addresses to the existing history

#### Scenario: Command summary unavailable for a tracked process
- **WHEN** command line metadata cannot be retrieved for a tracked PID
- **THEN** screen still shows PID and process name for that process without failing the render cycle

#### Scenario: Address list exceeds viewport height
- **WHEN** IPv4 and IPv6 entries require more rows than the address panel can display
- **THEN** system renders a scroll window of the address rows and displays the current visible range relative to total rows

#### Scenario: User scrolls downward through overflowed IPv4 addresses
- **WHEN** user presses `Down` while IPv4 overflow exists
- **THEN** system increases the IPv4 address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: User scrolls downward through overflowed IPv6 addresses
- **WHEN** user presses `j` or `J` while IPv6 overflow exists
- **THEN** system increases the IPv6 address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: User scrolls upward through overflowed IPv4 addresses
- **WHEN** user presses `Up` while IPv4 overflow exists
- **THEN** system decreases the IPv4 address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: User scrolls upward through overflowed IPv6 addresses
- **WHEN** user presses `k` or `K` while IPv6 overflow exists
- **THEN** system decreases the IPv6 address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: Scroll request exceeds bounds
- **WHEN** user scrolls above the first row or below the last row
- **THEN** system clamps the offset to valid bounds and keeps render stable without errors

#### Scenario: Terminal size or data size changes while scrolled
- **WHEN** terminal resize or list-size change makes the current offset invalid
- **THEN** system recomputes viewport capacity and clamps offset so the screen continues rendering valid rows

### Requirement: Stability indicator
The system SHALL display the elapsed time since the last new IP was discovered, helping the user judge when the whitelist is complete.

#### Scenario: No IPs discovered yet
- **WHEN** monitoring starts and no addresses have been discovered
- **THEN** the status bar shows "No IPs discovered yet"

#### Scenario: Recent IP discovery
- **WHEN** a new IP was discovered less than 60 seconds ago
- **THEN** the status bar shows "Last new: {seconds}s ago" with a live-updating elapsed counter

#### Scenario: No new IPs for over a minute
- **WHEN** 60 or more seconds have passed since the last new IP was discovered
- **THEN** the status bar shows "No new IPs for {minutes} min" with the elapsed minutes since last discovery

### Requirement: Export functionality
The system SHALL allow the user to export the current IP list to a file as plain text.

#### Scenario: User presses Export key
- **WHEN** user presses the Export hotkey (E)
- **THEN** system writes all discovered IPv4 subnets and canonical IPv6 `/64` entries to a text file (one per line) and displays the output file path

### Requirement: Copy to clipboard
The system SHALL allow the user to copy the current IP list to the system clipboard.

#### Scenario: User presses Copy key
- **WHEN** user presses the Copy hotkey (C)
- **THEN** system copies all discovered IPv4 subnets and canonical IPv6 `/64` entries (one per line) to the macOS clipboard

### Requirement: Navigation between screens
The system SHALL allow the user to return to the app selection screen from the monitoring screen without implicitly stopping monitoring for the current target, and SHALL persist discovered address history per target to a file in the command working directory for later restoration.

#### Scenario: User presses Escape during monitoring
- **WHEN** user presses Esc on the monitoring screen for target A
- **THEN** system returns to the app selection screen while target A continues monitoring in the background and keeps updating target A's collected address history

#### Scenario: User switches to a different target after going back
- **WHEN** user returns from target A and then selects target B
- **THEN** system starts or resumes monitoring target B in its own target session while target A remains monitored unless target A has been explicitly paused

#### Scenario: Monitoring data remains isolated by target
- **WHEN** user alternates between target A and target B while both have session data
- **THEN** each target view shows only that target's cached and live-discovered addresses and MUST NOT reuse the other target's addresses

### Requirement: Quit
The system SHALL exit cleanly when the user presses the quit key.

#### Scenario: User presses Quit key
- **WHEN** user presses Q
- **THEN** system stops all active monitoring sessions, persists per-target address history to the working-directory file, restores terminal state, and exits
