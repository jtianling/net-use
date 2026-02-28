## MODIFIED Requirements

### Requirement: Monitoring screen
The system SHALL display real-time monitoring status including tracked processes with PID, process name, and command summary when available, and discovered IPv4 and IPv6 addresses in switchable display and ordering modes.

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
