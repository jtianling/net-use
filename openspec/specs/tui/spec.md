# TUI

## Purpose

Provide an interactive terminal user interface for app selection, real-time monitoring of network connections, and export of discovered IP addresses.

## Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing discovered apps and running CLI processes, with text filtering support, and SHALL restore cached address history when a previously monitored target is selected again.

#### Scenario: User opens the tool without --pid/--name/--bundle
- **WHEN** user runs `sudo net-use` without target arguments
- **THEN** system displays the app selection screen with running GUI apps and running CLI processes listed first, then installed-only apps

#### Scenario: User types a filter string
- **WHEN** user types "chr" in the filter input
- **THEN** the list narrows to entries whose display name, Bundle ID, or PID text contains "chr" case-insensitively

#### Scenario: User selects an app
- **WHEN** user highlights an app and presses Enter
- **THEN** system transitions to the monitoring screen and begins monitoring the selected app

#### Scenario: User selects a running CLI process
- **WHEN** user highlights a CLI process entry and presses Enter
- **THEN** system starts monitoring that PID as the target and shows it in the monitoring screen header

#### Scenario: User reselects a previously monitored target
- **WHEN** user returns to the selection screen and selects a target that has cached history from the current session
- **THEN** system restores that target's previously discovered IPv4/IPv6 masked and raw address lists before appending newly collected results

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes with PID, process name, and command summary when available, and discovered IPv4 and IPv6 addresses in switchable display and ordering modes. When the address list exceeds the monitoring viewport, the system SHALL provide bounded vertical scrolling for the address region while keeping header, status, and footer context visible.

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
- **THEN** system toggles both IPv4 and IPv6 lists between discovery-time order and alphabetical order without losing collected data

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

#### Scenario: No new IPs for extended period
- **WHEN** 10 minutes have passed with no new IPs discovered
- **THEN** the status bar shows "No new IPs for 10 min" and the timestamp of the last discovery

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
The system SHALL allow the user to return to the app selection screen from the monitoring screen, and SHALL preserve discovered address history per target for later restoration.

#### Scenario: User presses Escape during monitoring
- **WHEN** user presses Esc on the monitoring screen
- **THEN** system stops monitoring, stores collected addresses under the current target identity, and returns to the app selection screen

#### Scenario: User switches to a different target after going back
- **WHEN** user returns from target A and then selects target B
- **THEN** system shows only target B's cached addresses (if any) and MUST NOT reuse target A's cached addresses in target B's monitoring view

### Requirement: Quit
The system SHALL exit cleanly when the user presses the quit key.

#### Scenario: User presses Quit key
- **WHEN** user presses Q
- **THEN** system stops monitoring, restores terminal state, and exits
