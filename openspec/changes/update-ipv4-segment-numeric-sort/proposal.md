## Why

Alphabetical sorting produces incorrect ordering for IPv4 addresses/subnets from a network perspective (for example, `100.0.0.0` appears before `9.0.0.0`). This makes the monitoring view harder to scan and compare.

## What Changes

- Change IPv4 ordering behavior in TUI order mode from lexical/alphabetical comparison to numeric-by-octet comparison.
- Keep the existing order-mode toggle behavior, but redefine the non-discovery mode for IPv4 as numeric segment ordering.
- Keep IPv6 ordering behavior unchanged.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: Update monitoring-screen address ordering requirements so IPv4 entries are sorted by octet numeric value rather than alphabetical string order.

## Impact

- Affected spec: `openspec/specs/tui/spec.md` (delta spec required).
- Likely affected code: IPv4 ordering logic in TUI monitor view.
- No API or dependency changes expected.
