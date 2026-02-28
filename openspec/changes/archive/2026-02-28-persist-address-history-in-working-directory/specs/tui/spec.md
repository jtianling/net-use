## MODIFIED Requirements

### Requirement: App selection screen
The system SHALL present a TUI screen listing discovered apps and running CLI processes, with text filtering support, and SHALL restore persisted address history from the command working directory when a previously monitored target is selected again.

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

#### Scenario: User reselects a previously monitored target after restart
- **WHEN** user starts `net-use` in a working directory that contains persisted history and selects a target with saved records
- **THEN** system restores that target's previously discovered IPv4/IPv6 masked and raw address lists before appending newly collected results

#### Scenario: Persisted history file is missing or invalid
- **WHEN** system cannot read or parse the persisted history file in the working directory
- **THEN** system continues to app selection and monitoring with empty history without crashing

### Requirement: Navigation between screens
The system SHALL allow the user to return to the app selection screen from the monitoring screen, and SHALL persist discovered address history per target to a file in the command working directory for later restoration.

#### Scenario: User presses Escape during monitoring
- **WHEN** user presses Esc on the monitoring screen
- **THEN** system stops monitoring, stores collected addresses under the current target identity, writes updated history to the working-directory persistence file, and returns to the app selection screen

#### Scenario: User switches to a different target after going back
- **WHEN** user returns from target A and then selects target B
- **THEN** system shows only target B's cached addresses (if any) and MUST NOT reuse target A's cached addresses in target B's monitoring view
