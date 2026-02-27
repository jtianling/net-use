## ADDED Requirements

### Requirement: Locate target process by PID
The system SHALL accept a numeric PID as input and begin monitoring that process immediately.

#### Scenario: Valid PID provided
- **WHEN** user provides `--pid 1234` and process 1234 exists
- **THEN** system begins monitoring process 1234 and its child processes

#### Scenario: PID does not exist
- **WHEN** user provides `--pid 9999` and no such process exists
- **THEN** system exits with an error message indicating the process was not found

### Requirement: Locate target process by name
The system SHALL accept a process name and find matching running processes.

#### Scenario: Process name matches a running process
- **WHEN** user provides `--name "curl"` and a process named "curl" is running
- **THEN** system begins monitoring the matched process and its child processes

#### Scenario: Multiple processes match the name
- **WHEN** user provides `--name "python"` and multiple python processes are running
- **THEN** system monitors all matching processes and their child processes

#### Scenario: No process matches the name
- **WHEN** user provides `--name "nonexistent"` and no matching process is running
- **THEN** system waits for a process with that name to appear, polling periodically

### Requirement: Locate target process by Bundle ID
The system SHALL accept a macOS Bundle ID and resolve it to running process(es).

#### Scenario: Bundle ID matches a running app
- **WHEN** user provides `--bundle com.google.Chrome` and Chrome is running
- **THEN** system resolves the bundle's executable name via Info.plist, finds matching processes by executable path, and begins monitoring

#### Scenario: Bundle ID app is not running
- **WHEN** user provides `--bundle com.google.Chrome` and Chrome is not running
- **THEN** system waits for the app to launch, polling periodically, and begins monitoring when detected

### Requirement: Track entire child process tree
The system SHALL recursively discover and monitor all descendant processes of the target.

#### Scenario: App spawns new child processes during monitoring
- **WHEN** Chrome opens a new tab (creating a new Renderer process)
- **THEN** the new child process is detected within the next polling cycle and added to the monitored set

#### Scenario: Child process exits
- **WHEN** a monitored child process terminates
- **THEN** system removes it from the active tracking set without affecting collected IP data

### Requirement: Handle app restart
The system SHALL continue monitoring across app restarts.

#### Scenario: Monitored app exits and relaunches
- **WHEN** the target app exits and later restarts
- **THEN** system detects the new instance, resumes monitoring, and preserves all previously collected IP data
