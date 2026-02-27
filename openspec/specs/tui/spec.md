# TUI

## Purpose

Provide an interactive terminal user interface for app selection, real-time monitoring of network connections, and export of discovered IP addresses.

## Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing all discovered apps (running and installed-only) with text filtering support.

#### Scenario: User opens the tool without --pid/--name/--bundle
- **WHEN** user runs `sudo net-use` without target arguments
- **THEN** system displays the app selection screen with running apps listed first, then installed-only apps

#### Scenario: User types a filter string
- **WHEN** user types "chr" in the filter input
- **THEN** the list narrows to apps whose name or Bundle ID contains "chr" (case-insensitive)

#### Scenario: User selects an app
- **WHEN** user highlights an app and presses Enter
- **THEN** system transitions to the monitoring screen and begins monitoring the selected app

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes and discovered IP subnets/addresses.

#### Scenario: Monitoring an active app
- **WHEN** monitoring is active
- **THEN** screen shows: target app name and Bundle ID, monitoring status and uptime, list of tracked PIDs with process names, list of discovered IPv4 subnets (/24), list of discovered IPv6 addresses

#### Scenario: New IP subnet discovered
- **WHEN** a new IPv4 subnet or IPv6 address is discovered
- **THEN** it appears in the list immediately on the next render cycle

### Requirement: Stability indicator
The system SHALL display the elapsed time since the last new IP was discovered, helping the user judge when the whitelist is complete.

#### Scenario: No new IPs for extended period
- **WHEN** 10 minutes have passed with no new IPs discovered
- **THEN** the status bar shows "No new IPs for 10 min" and the timestamp of the last discovery

### Requirement: Export functionality
The system SHALL allow the user to export the current IP list to a file as plain text.

#### Scenario: User presses Export key
- **WHEN** user presses the Export hotkey (E)
- **THEN** system writes all discovered subnets/addresses to a text file (one per line) and displays the output file path

### Requirement: Copy to clipboard
The system SHALL allow the user to copy the current IP list to the system clipboard.

#### Scenario: User presses Copy key
- **WHEN** user presses the Copy hotkey (C)
- **THEN** system copies all discovered subnets/addresses (one per line) to the macOS clipboard

### Requirement: Navigation between screens
The system SHALL allow the user to return to the app selection screen from the monitoring screen.

#### Scenario: User presses Escape during monitoring
- **WHEN** user presses Esc on the monitoring screen
- **THEN** system stops monitoring, preserves collected data, and returns to the app selection screen

### Requirement: Quit
The system SHALL exit cleanly when the user presses the quit key.

#### Scenario: User presses Quit key
- **WHEN** user presses Q
- **THEN** system stops monitoring, restores terminal state, and exits
