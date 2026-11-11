## Why

Slotcar fingerprints already borrow Unison's content-addressed identity idea. The next review-oriented step is to make review labels, policy names, evidence handles, and fingerprint manifests resolve through a name-to-hash registry so mutable human names do not become the durable source of truth.

## What Changes

- Add a review identity registry contract where human labels point to immutable reviewed definitions or evidence hashes.
- Require fingerprint review manifests to preserve both mutable label and immutable identity fields.
- Define rollback/history behavior for review-label pointer updates.
- Document that hashes anchor evidence identity while names remain ergonomic aliases.

## Non-Goals

- Do not require blanket fingerprints for every function.
- Do not replace Git history or OpenSpec archives.
- Do not claim a matching review identity proves code correctness.

## Capabilities

### New Capabilities
- `slotcar-review-identity-registry`: Name-to-hash registry for review labels, evidence handles, and fingerprint manifests.

### Modified Capabilities
- `standards-codification`: Extends function fingerprint metadata and review manifest behavior.
- `documentation`: Adds operator guidance for review identity labels and rollback.

## Impact

- **Files**: future review manifest schema, fingerprint output, docs, tests, and registry validation.
- **APIs**: new immutable identity fields and pointer-history inspection.
- **Testing**: manifest hash stability, label update, rollback, and stale-label diagnostics.
