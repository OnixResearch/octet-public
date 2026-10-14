## Why

Octet is the canonical product identity, but source-attribute namespaces and emitted group metadata still show mixed state. Consumer docs and templates describe `octet::...` attrs, while current code and fixtures still use `tigerstyle::...` groups in several places. That mismatch can confuse consumers, break suppressions, and make runner output look inconsistent with product positioning.

The project needs a deliberate namespace contract. For this release line, the safe contract is to explicitly defer source-attribute migration and make docs state the live `tigerstyle::` behavior.

## What Changes

- Record `tigerstyle::` as the canonical source-attribute namespace and emitted group namespace for current releases.
- Implement code, UI fixtures, docs, templates, runner metadata, and snapshots consistently with that decision.
- Preserve tested compatibility aliases for supported legacy namespaces or document their deprecation boundary.
- Add migration guidance for consumers using legacy `tigerstyle::...` or `slotcar::...` attrs/config names.

## Impact

- **Files**: lint declarations under `src/**`, `src/diagnostics.rs`, UI fixtures, `cargo-octet/src/output.rs`, README, `docs/lint-reference.md`, `docs/consumer-guide.md`, `templates/dylint.toml`, and snapshots.
- **Testing**: source-attr and group-attr UI fixtures for canonical and compatibility namespaces, config precedence tests, runner JSON/SARIF metadata tests, and docs freshness tests.
