## Why

In the current TUI flow, leaving a monitoring screen stops collection for that app immediately. This makes cross-app investigation inefficient because users must keep restarting monitoring when switching targets.

## What Changes

- Keep monitoring active for a selected app even after leaving its monitoring screen.
- Allow users to switch to another app and start monitoring it while previously selected apps continue collecting in the same CLI session.
- Introduce an explicit pause action (`p`) inside a target's monitoring view to stop that target's ongoing monitoring.
- Update screen-navigation behavior so `Esc` returns to selection without implicitly stopping monitoring.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `tui`: change monitoring lifecycle semantics so monitoring is persistent across screen transitions and only stops for a target when the user explicitly pauses it.

## Impact

- Affected code: TUI state model, monitor session lifecycle management, key handling, and monitor-view status rendering.
- No external API change expected.
- No new external dependency expected.
