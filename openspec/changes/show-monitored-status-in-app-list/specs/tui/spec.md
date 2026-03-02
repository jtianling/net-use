## MODIFIED Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing discovered apps and running CLI processes, with text filtering support, and SHALL restore persisted address history from the command working directory when a previously monitored target is selected again. The selector SHALL allow re-entering an already-active target monitor session without discarding in-memory collected data for that target. The selector SHALL also show each row's current monitoring lifecycle state for the current session.

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

#### Scenario: Selector marks actively monitored targets
- **WHEN** target A has an active monitor session and user returns to the app selection screen
- **THEN** target A's row shows a monitoring-state indicator representing active monitoring

#### Scenario: Selector marks paused and unmonitored targets distinctly
- **WHEN** target A is paused and target B has no session in the current run
- **THEN** target A and target B rows show different monitoring-state indicators so paused and never-monitored states are distinguishable
