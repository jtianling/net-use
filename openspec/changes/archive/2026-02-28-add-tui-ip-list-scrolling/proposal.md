## Why

When a process has many IPv4/IPv6 addresses, the current TUI list can exceed one screen and hide entries. Operators need a predictable way to scroll through the full list without losing context.

## What Changes

- Add vertical scrolling behavior for the TUI IP address area when content exceeds the visible viewport.
- Add keyboard navigation for moving the IP list viewport up/down by line and by page.
- Keep header/footer and process context stable while only the address list region scrolls.
- Define visible scroll indicators (position and bounds feedback) so users know more content exists.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: extend requirements so IPv4/IPv6 address sections support bounded vertical scrolling and keyboard-driven navigation when entries overflow the screen.

## Impact

- Affected code: TUI rendering/layout code, input handling for key events, and list state management.
- No external API changes expected.
- No new runtime dependencies expected; behavior should fit existing terminal UI framework.
