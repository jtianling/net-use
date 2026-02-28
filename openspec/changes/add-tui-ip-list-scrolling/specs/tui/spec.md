## MODIFIED Requirements

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes with PID, process name, and command summary when available, and discovered IPv4 and IPv6 addresses in a switchable display mode. When the address list exceeds the monitoring viewport, the system SHALL provide bounded vertical scrolling for the address region while keeping header, status, and footer context visible.

#### Scenario: Monitoring an active app
- **WHEN** monitoring is active
- **THEN** screen shows: target app name and Bundle ID, monitoring status and uptime, list of tracked PIDs with process names and command summary when present, list of discovered IPv4 subnets (/24), list of discovered IPv6 subnets (/64), with masked mode enabled by default

#### Scenario: New IP subnet discovered
- **WHEN** a new IPv4 subnet or IPv6 `/64` subnet is discovered
- **THEN** it appears in the list immediately on the next render cycle

#### Scenario: User toggles address display mode
- **WHEN** user presses `S` on the monitoring screen
- **THEN** system toggles both IPv4 and IPv6 lists between masked and raw full-address display without losing collected data

#### Scenario: Command summary unavailable for a tracked process
- **WHEN** command line metadata cannot be retrieved for a tracked PID
- **THEN** screen still shows PID and process name for that process without failing the render cycle

#### Scenario: Address list exceeds viewport height
- **WHEN** IPv4 and IPv6 entries require more rows than the address panel can display
- **THEN** system renders a scroll window of the address rows and displays the current visible range relative to total rows

#### Scenario: User scrolls downward through overflowed addresses
- **WHEN** user presses a downward scroll key (`Down`, `j`, `PageDown`, or `Space`) while overflow exists
- **THEN** system increases the address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: User scrolls upward through overflowed addresses
- **WHEN** user presses an upward scroll key (`Up`, `k`, or `PageUp`) while overflow exists
- **THEN** system decreases the address-list offset within bounds and updates visible rows on the next render cycle

#### Scenario: Scroll request exceeds bounds
- **WHEN** user scrolls above the first row or below the last row
- **THEN** system clamps the offset to valid bounds and keeps render stable without errors

#### Scenario: Terminal size or data size changes while scrolled
- **WHEN** terminal resize or list-size change makes the current offset invalid
- **THEN** system recomputes viewport capacity and clamps offset so the screen continues rendering valid rows
