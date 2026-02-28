## Context

The monitoring view currently supports toggling address order mode between discovery-time and alphabetical order. For IPv4 strings, alphabetical sorting does not reflect numeric network order (for example, `100.0.0.0` sorts before `9.0.0.0`). This change updates ordering semantics only for IPv4 while preserving existing IPv6 behavior and key bindings.

## Goals / Non-Goals

**Goals:**
- Define deterministic IPv4 sorting by octet numeric value in non-discovery order mode.
- Preserve current discovery-time order mode behavior.
- Preserve current IPv6 ordering behavior in non-discovery mode.
- Keep UI interaction and persistence behavior unchanged.

**Non-Goals:**
- Changing IPv6 ordering semantics.
- Introducing new order modes or key bindings.
- Modifying address collection, masking rules, or export/copy formats.

## Decisions

1. Use numeric tuple comparison for IPv4 order mode.
- Decision: Convert IPv4 text into four octets and compare as `(o1, o2, o3, o4)` ascending.
- Rationale: This directly matches user expectation for subnet/address magnitude and avoids lexical anomalies.
- Alternative considered: Zero-padding string segments before lexical sort. Rejected because it adds string-normalization complexity and is less explicit than numeric comparison.

2. Keep IPv6 non-discovery sorting lexical.
- Decision: Retain current IPv6 alphabetical sort behavior when order mode is toggled.
- Rationale: The requested change targets IPv4 only; preserving IPv6 minimizes scope and regression risk.
- Alternative considered: Introduce numeric/structural IPv6 sort. Rejected as out-of-scope and unnecessary for this request.

3. Keep mixed-mode safety behavior stable.
- Decision: If parsing fails unexpectedly for an IPv4 entry, fallback to stable string comparison for that entry path rather than panicking.
- Rationale: Defensive handling protects UI rendering from malformed input and keeps behavior deterministic.
- Alternative considered: Hard-fail or drop malformed entries. Rejected to avoid user-visible instability.

## Risks / Trade-offs

- [Risk] Parsing overhead on every render sort pass. -> Mitigation: Parse only during sorting pass and keep algorithm simple (fixed four-octet parse).
- [Risk] Behavior differences for unexpected non-IPv4 text in IPv4 list. -> Mitigation: Define string-comparison fallback to preserve stable output ordering.
- [Trade-off] IPv4 and IPv6 use different non-discovery ordering rules. -> Mitigation: Document explicitly in spec and keep UI label generic to avoid keybinding churn.

## Migration Plan

- No data migration required.
- Implement sorting logic change and update tests for ordering expectations.
- Rollback path: revert sorting comparator change; no persisted format changes involved.

## Open Questions

- None currently; requested behavior is explicit for IPv4 and intentionally leaves IPv6 unchanged.
