## Why

Octet now publishes the central Verus toolchain packages, but its compatibility inventory names only Octet, Trellis, VerusBelt, and an unsupported formatter target. OnixOS and Aspen cannot fail-closed against reviewed consumer rows until their exact package-consumption scopes are registered.

## What Changes

- Register OnixOS lifecycle proof execution and Aspen's node-replication probe as exact native package consumers.
- Restrict compatibility consumer identifiers to the reviewed inventory.
- Add a negative fixture for an unregistered consumer.
- Preserve explicit non-transfer wording for both new rows.

## Impact

- **Files**: the Nickel profile, contract, generated JSON, fixtures, policy registry, README, and this Cairn change.
- **Testing**: profile freshness, positive and negative Nickel fixtures, formatter package checks, policy registry, and Cairn gates.
