## 1. IPv4 Ordering Logic

- [x] 1.1 Locate the TUI non-discovery sorting path for IPv4/IPv6 entries in `src/tui/monitor_view.rs` and isolate IPv4 sort comparison from shared lexical ordering.
- [x] 1.2 Implement IPv4 comparator that parses dot-separated octets and sorts ascending by numeric tuple `(o1, o2, o3, o4)`.
- [x] 1.3 Preserve IPv6 non-discovery ordering behavior as existing alphabetical sort and keep discovery-time mode behavior unchanged.
- [x] 1.4 Add defensive fallback for unexpected IPv4 parse failures to keep ordering deterministic and avoid render-time panics.

## 2. Tests

- [x] 2.1 Update/add unit tests for address ordering mode to assert IPv4 numeric order (including `9.0.0.0` before `100.0.0.0`).
- [x] 2.2 Keep or add assertions that IPv6 sorted mode remains alphabetical and unaffected by the IPv4 comparator change.
- [x] 2.3 Run relevant test targets and confirm no regressions in monitor view ordering behavior.

## 3. Verification

- [x] 3.1 Manually validate monitor screen order toggle (`O`) with mixed IPv4 values and confirm discovery-time mode still preserves arrival order.
- [x] 3.2 Verify raw and masked display modes (`S`) both apply the updated IPv4 numeric sorted mode correctly.
- [x] 3.3 Update change notes/comments in code where needed to document IPv4 numeric ordering intent.
