## MODIFIED Requirements

### Requirement: Aggregate IPv4 addresses to /24 subnets
The system SHALL mask IPv4 addresses by zeroing the last octet and formatting as `x.x.x.0/24` for canonical output and deduplication, while retaining full IPv4 addresses for raw-display state.

#### Scenario: New IPv4 address in a new /24 subnet
- **WHEN** system encounters remote IP `142.250.80.37` and `142.250.80.0/24` has not been seen before
- **THEN** system records `142.250.80.0/24` as a new canonical subnet and notifies the output pipeline

#### Scenario: New IPv4 address in an already-seen /24 subnet
- **WHEN** system encounters remote IP `142.250.80.99` and `142.250.80.0/24` is already recorded
- **THEN** system does not produce a duplicate canonical output entry

#### Scenario: Distinct raw IPv4 addresses under the same /24
- **WHEN** system encounters `142.250.80.37` and later `142.250.80.99`, and both are previously unseen full addresses
- **THEN** system keeps both full addresses available for raw-address display mode

### Requirement: Record full IPv6 addresses
The system SHALL normalize IPv6 addresses to `/64` for canonical output and deduplication, while retaining full IPv6 addresses for raw-display state.

#### Scenario: New IPv6 address in a new /64 subnet
- **WHEN** system encounters remote IPv6 `2607:6bc0::10` and `2607:6bc0::/64` has not been seen before
- **THEN** system records `2607:6bc0::/64` as a new canonical entry and notifies the output pipeline

#### Scenario: New IPv6 address in an already-seen /64 subnet
- **WHEN** system encounters remote IPv6 `2607:6bc0::11` and `2607:6bc0::/64` is already recorded
- **THEN** system does not produce a duplicate canonical output entry

#### Scenario: Distinct raw IPv6 addresses under the same /64
- **WHEN** system encounters `2607:6bc0::10` and later `2607:6bc0::11`, and both are previously unseen full addresses
- **THEN** system keeps both full addresses available for raw-address display mode
