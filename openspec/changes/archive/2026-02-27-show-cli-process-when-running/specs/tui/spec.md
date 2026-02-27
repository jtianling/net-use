## MODIFIED Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing discovered apps and running CLI processes, with text filtering support.

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

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes with PID, process name, and command summary when available, and discovered IPv4 and IPv6 addresses in a switchable display mode.

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
