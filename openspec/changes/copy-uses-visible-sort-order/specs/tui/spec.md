## MODIFIED Requirements

### Requirement: Copy to clipboard
The system SHALL allow the user to copy the current IP list to the system clipboard, and the copied entry order SHALL match the order currently shown on the monitoring screen.

#### Scenario: User presses Copy key
- **WHEN** user presses the Copy hotkey (C)
- **THEN** system copies the currently displayed IPv4 and IPv6 entries (one per line) to the macOS clipboard, matching the active display mode

#### Scenario: User copies after switching order mode
- **WHEN** user presses `O` on the monitoring screen to switch to deterministic sorted order and then presses `C`
- **THEN** the clipboard content uses the same sorted IPv4/IPv6 order currently displayed on screen rather than discovery-time order

#### Scenario: User copies after switching display mode
- **WHEN** user presses `S` on the monitoring screen to switch between masked and raw views and then presses `C`
- **THEN** the clipboard content uses the same entry type (masked subnets or raw addresses) currently displayed on screen
