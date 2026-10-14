## Why

Octet now has a tiny production-mode Verus pilot, but the pilot is only proven on a successful proof path. That is not enough evidence for a production formal-proof rail: reviewers also need to see that an intentionally invalid Verus proof fails closed and cannot create `formal_proof` provenance.

This change adds a negative production proof case before expanding the proof surface. The goal is to make failure semantics observable, deterministic, and scoped without weakening the existing fixture rail or ordinary `cargo octet check` starter path.

## What Changes

- Add an Octet-owned failing Verus proof fixture that real Verus rejects deterministically.
- Add a production-mode negative rail or focused harness path that exercises `octet-production-verus` against that failing proof.
- Ensure failed production proof execution records bounded failure artifacts and deterministic diagnostics, but emits no `formal_proof` provenance.
- Add tests proving both direct wrapper failure and `cargo octet prove` fail-closed behavior for the selected negative rail.
- Keep trust-boundary wording explicit that the negative case tests fail-closed behavior, not a new correctness claim.

## Non-Goals

- Do not broaden production proof claims beyond the existing tiny pilot.
- Do not make ordinary `cargo octet check` depend on Verus.
- Do not replace the fixture rail or use the fixture verifier as production evidence.
- Do not accept ambient, unpinned Verus or solver binaries.
- Do not add broad semantic verification for Octet internals in this change.

## Impact

- **Files**: `verus/proofs/`, `standards/proof-rails/manifest.ncl`, `standards/policies/registry.ncl`, `cargo-octet`/`octet-standards` proof tests, docs as needed.
- **Behavior**: production proof failure becomes an explicitly tested fail-closed path with no `formal_proof` provenance.
- **Testing**: direct `octet-production-verus` negative proof run, `cargo octet prove` negative rail run, artifact verification/provenance assertions, existing positive production rail regression.
