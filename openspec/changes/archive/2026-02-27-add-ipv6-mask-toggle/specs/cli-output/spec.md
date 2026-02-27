## MODIFIED Requirements

### Requirement: No-TUI output mode
The system SHALL support a `--no-tui` flag that outputs discovered IPs to stdout without any TUI rendering.

#### Scenario: New IP discovered in no-tui mode
- **WHEN** `--no-tui` is active and a new IPv4 subnet is discovered
- **THEN** system prints `x.x.x.0/24` to stdout immediately, one line per subnet

#### Scenario: New IPv6 address discovered in no-tui mode
- **WHEN** `--no-tui` is active and a new IPv6 address is discovered
- **THEN** system prints the canonical IPv6 `/64` subnet format (for example `2607:6bc0::/64`) to stdout immediately, one line per subnet

#### Scenario: Duplicate IP in no-tui mode
- **WHEN** an already-recorded subnet or address is encountered again
- **THEN** system produces no output for the duplicate
