# Connection Collector

## Purpose

Collect and deduplicate remote IP addresses from monitored processes by inspecting their socket file descriptors, aggregating IPv4 addresses to /24 subnets and recording full IPv6 addresses.

## Requirements

### Requirement: Collect remote IP addresses from socket file descriptors
The system SHALL use `proc_pidfdinfo` to enumerate socket file descriptors of each monitored process and extract the remote IP address from TCP and UDP connections.

#### Scenario: Process has active TCP connections
- **WHEN** a monitored process has open TCP sockets connected to remote hosts
- **THEN** system extracts the remote IP address from each socket

#### Scenario: Process has active UDP sockets
- **WHEN** a monitored process has UDP sockets with known remote addresses
- **THEN** system extracts the remote IP address from each socket

#### Scenario: Process has no network connections
- **WHEN** a monitored process has no socket file descriptors
- **THEN** system continues polling without producing output

### Requirement: Aggregate IPv4 addresses to /24 subnets
The system SHALL mask IPv4 addresses by zeroing the last octet and formatting as `x.x.x.0/24`.

#### Scenario: New IPv4 address in a new /24 subnet
- **WHEN** system encounters remote IP `142.250.80.37` and `142.250.80.0/24` has not been seen before
- **THEN** system records `142.250.80.0/24` as a new subnet and notifies the UI

#### Scenario: New IPv4 address in an already-seen /24 subnet
- **WHEN** system encounters remote IP `142.250.80.99` and `142.250.80.0/24` is already recorded
- **THEN** system does not produce a duplicate entry

### Requirement: Record full IPv6 addresses
The system SHALL record IPv6 addresses in their complete form without masking.

#### Scenario: New IPv6 address
- **WHEN** system encounters remote IPv6 `2607:f8b0:4004:0800::200e` not previously seen
- **THEN** system records the full address and notifies the UI

#### Scenario: Duplicate IPv6 address
- **WHEN** system encounters an IPv6 address already recorded
- **THEN** system does not produce a duplicate entry

### Requirement: Ignore loopback and link-local addresses
The system SHALL exclude loopback (127.0.0.0/8, ::1) and link-local (169.254.0.0/16, fe80::/10) addresses from output.

#### Scenario: Loopback connection detected
- **WHEN** a socket's remote address is 127.0.0.1 or ::1
- **THEN** system does not record it

#### Scenario: Link-local connection detected
- **WHEN** a socket's remote address is in 169.254.0.0/16 or fe80::/10
- **THEN** system does not record it

### Requirement: Polling interval
The system SHALL poll at approximately 200ms intervals.

#### Scenario: Continuous monitoring
- **WHEN** monitoring is active
- **THEN** system completes a full poll cycle (process tree update + socket enumeration) approximately every 200ms
