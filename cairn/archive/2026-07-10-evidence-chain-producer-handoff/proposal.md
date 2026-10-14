## Why

The function-address path is only the first evidence profile that needs stack-wide content identity. Octet should expose a general producer-handoff contract so future rails such as lint receipts, proof receipts, dependency audits, and state-machine evidence can enter the same Kamacite → Valence → Mantle → Cairn chain without inventing profile-specific wrappers each time.

## What Changes

- Define a generic Octet evidence-chain producer handoff envelope.
- Require typed evidence kind, producer identity, canonical roots, parent links, policy/profile hashes, projections, and non-claim boundaries.
- Keep profile-specific payloads such as function-address records under profile-owned extensions.
- Add positive and negative fixtures using function-address as the first concrete profile.

## Impact

Octet remains a source/proof/check producer while Kamacite owns canonical portable receipts. New Octet evidence rails can join the stack evidence chain incrementally without making JSON the durable authority.
