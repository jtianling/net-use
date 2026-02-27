## MODIFIED Requirements

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes, discovered IPv4 and IPv6 addresses in a switchable display mode.

#### Scenario: Monitoring an active app
- **WHEN** monitoring is active
- **THEN** screen shows: target app name and Bundle ID, monitoring status and uptime, list of tracked PIDs with process names, list of discovered IPv4 subnets (/24), list of discovered IPv6 subnets (/64), with masked mode enabled by default

#### Scenario: New IP subnet discovered
- **WHEN** a new IPv4 subnet or IPv6 `/64` subnet is discovered
- **THEN** it appears in the list immediately on the next render cycle

#### Scenario: User toggles address display mode
- **WHEN** user presses `S` on the monitoring screen
- **THEN** system toggles both IPv4 and IPv6 lists between masked and raw full-address display without losing collected data

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
