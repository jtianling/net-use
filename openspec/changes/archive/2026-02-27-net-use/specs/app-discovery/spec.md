## ADDED Requirements

### Requirement: Discover installed applications
The system SHALL scan `/Applications` and `~/Applications` directories for `.app` bundles and extract metadata from each bundle's `Info.plist`.

#### Scenario: Standard app installed in /Applications
- **WHEN** `/Applications/Slack.app` exists with a valid `Info.plist`
- **THEN** system extracts Bundle ID (`CFBundleIdentifier`), display name (`CFBundleName` or `CFBundleDisplayName`), and executable name (`CFBundleExecutable`)

#### Scenario: App installed in ~/Applications
- **WHEN** `~/Applications/MyApp.app` exists with a valid `Info.plist`
- **THEN** system includes it in the discovered app list

#### Scenario: Invalid or corrupt app bundle
- **WHEN** an `.app` directory lacks a valid `Info.plist` or required keys
- **THEN** system skips it without error

### Requirement: Discover running processes
The system SHALL enumerate currently running processes and identify those that correspond to known app bundles.

#### Scenario: Running GUI app
- **WHEN** Google Chrome is running
- **THEN** system lists it as a running app with its PID, display name, and Bundle ID

#### Scenario: Running CLI process
- **WHEN** a CLI process like `curl` is running
- **THEN** system lists it as a running process with its PID and process name

### Requirement: Distinguish running vs installed-only apps
The system SHALL indicate whether each discovered app is currently running or only installed.

#### Scenario: App list display
- **WHEN** user views the app list
- **THEN** running apps are marked distinctly (e.g., with a status indicator) and show their PID, installed-only apps show no PID
