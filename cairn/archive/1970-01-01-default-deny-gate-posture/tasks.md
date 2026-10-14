## Phase 1: Implementation

- [x] [serial] r[octet.default_deny_gate] Add `docs/default-deny-gate-posture.md` posture documentation: default-on, bypass-labeled, stable-exit-code, byte-deterministic.
- [x] [serial] r[octet.default_deny_gate] Add `templates/cargo-alias.toml`: copyable cargo alias making `cargo octet-check` the default-on checked build path.
- [x] [serial] r[octet.default_deny_gate] Add `templates/ci-gate.yml`: copyable CI step running the gate with SARIF output and artifact verification.

## Phase 2: Archive

- [x] [serial] r[octet.default_deny_gate] Verify and archive the change.

## Verification Coverage

| Scenario | Evidence |
|---|---|
| Posture doc exists | `docs/default-deny-gate-posture.md` created |
| Cargo alias template exists | `templates/cargo-alias.toml` created |
| CI gate template exists | `templates/ci-gate.yml` created |
| Documentation/template-only change | no `.rs` files changed |
