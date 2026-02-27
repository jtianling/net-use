## ADDED Requirements

### Requirement: No-TUI output mode
The system SHALL support a `--no-tui` flag that outputs discovered IPs to stdout without any TUI rendering.

#### Scenario: New IP discovered in no-tui mode
- **WHEN** `--no-tui` is active and a new IPv4 subnet is discovered
- **THEN** system prints `x.x.x.0/24` to stdout immediately, one line per subnet

#### Scenario: New IPv6 address discovered in no-tui mode
- **WHEN** `--no-tui` is active and a new IPv6 address is discovered
- **THEN** system prints the full IPv6 address to stdout immediately, one line per address

#### Scenario: Duplicate IP in no-tui mode
- **WHEN** an already-recorded subnet or address is encountered again
- **THEN** system produces no output for the duplicate

### Requirement: Pipe-friendly output
The system SHALL produce clean output suitable for piping to files or other commands.

#### Scenario: Redirect to file
- **WHEN** user runs `sudo net-use --pid 1234 --no-tui > ips.txt`
- **THEN** the file contains only IP subnets/addresses, one per line, with no status messages, progress indicators, or decorative output

#### Scenario: Ctrl+C terminates cleanly
- **WHEN** user sends SIGINT (Ctrl+C) during no-tui mode
- **THEN** system exits immediately without printing additional output

### Requirement: CLI target specification is required in no-tui mode
The system SHALL require at least one of `--pid`, `--name`, or `--bundle` when `--no-tui` is used.

#### Scenario: No target specified with --no-tui
- **WHEN** user runs `sudo net-use --no-tui` without a target
- **THEN** system exits with an error message indicating a target is required in no-tui mode
