## Why

Unison separates stable content identity from mutable names and handles refactoring as a structured session rather than a pile of text edits. Octet already has fingerprints, namespace compatibility rules, and a basic migration manifest command, but larger product, lint, policy, or evidence-surface migrations still need a first-class session model that records old names, new names, identity links, compatibility aliases, todos, validation commands, and rollback boundaries.

## What Changes

- Extend Octet migration support into structured migration sessions for namespace changes, lint/policy renames, evidence schema transitions, and artifact layout moves.
- Record name-pointer mappings separately from content identity: old symbol or surface name, new name, BLAKE3-backed object/artifact identity where available, compatibility classification, and removal boundary.
- Add plan/status/verify output that tracks affected docs, configs, source attributes, receipts, transcripts, and generated artifacts without mutating files implicitly.
- Preserve compatibility and evidence boundaries: migration receipts prove planned/refreshed identity linkage and replayed checks, not semantic equivalence or correctness.

## Impact

- **Files / artifacts**: expected implementation areas are `cargo-octet/src/` migration code, docs/examples, templates for migration plans, policy registry entries for migration receipts, and focused fixtures.
- **Ordering / dependencies**: define session DTOs and receipt schema before CLI mutation support; add verification before any broad sweep support.
- **Testing**: positive rename/session fixtures, negative stale hash/missing alias/unsupported removal-boundary fixtures, idempotence checks for dry-run/apply/status/verify, and docs-example transcripts.
- **Compatibility**: existing `cargo octet migrate start --manifest ...` behavior remains available; new session modes are additive until promoted.
