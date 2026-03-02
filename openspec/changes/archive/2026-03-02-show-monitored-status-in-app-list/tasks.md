## 1. Data Model And State Derivation

- [x] 1.1 Add selector-facing monitoring state to app row model (`AppInfo`) with values for active, paused, and unmonitored.
- [x] 1.2 Update TUI list preparation flow to derive each app row's monitoring state from existing session registry before building `AppSelector`.

## 2. Selector Rendering

- [x] 2.1 Update `AppSelector` row rendering to display monitoring-state label for every app/process entry.
- [x] 2.2 Keep existing filter/sort behavior unchanged while integrating the new monitoring-state label.

## 3. Verification

- [x] 3.1 Add or update unit tests covering displayed state for active, paused, and unmonitored targets in selector rows.
- [x] 3.2 Run project tests to verify no regressions in TUI selection and monitor resume flow.
