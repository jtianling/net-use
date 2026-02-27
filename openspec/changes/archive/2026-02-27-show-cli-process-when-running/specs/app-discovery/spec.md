## MODIFIED Requirements

### Requirement: Discover running processes
The system SHALL enumerate currently running processes and include both GUI apps that map to known app bundles and standalone CLI processes that have no bundle mapping.

#### Scenario: Running GUI app
- **WHEN** Google Chrome is running
- **THEN** system lists it as a running app with its PID, display name, and Bundle ID

#### Scenario: Running CLI process
- **WHEN** a CLI process like `curl` is running
- **THEN** system lists it as a running process with its PID and process name, and leaves Bundle ID empty

#### Scenario: Multiple CLI processes share the same name
- **WHEN** two `python` processes are running with different PIDs
- **THEN** system lists both entries separately and preserves each PID for later selection

#### Scenario: Process metadata cannot be resolved
- **WHEN** a running PID cannot provide a valid name or path
- **THEN** system skips that PID and continues enumerating the remaining processes
