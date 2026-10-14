## Phase 1: Implementation

- [x] I1 Add separate fresh-build cargo-octet package producers and a named flake check.
- [x] I2 Compare fresh-build generated receipts against each other and the expanded corpus fixture while preserving boundary assertions.
- [x] I3 Document the named fresh-build replay check and bounded non-proof scope in the contributor guide.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Fresh-build binaries reproduce expanded corpus receipt" with `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L`; save output at `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/fresh-build-object-corpus-check.txt`.
- [x] V2 Verify scenario "Fresh-build rail preserves bounded non-proof claims" with the same check output; save output at `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/fresh-build-object-corpus-check.txt`.
- [x] V3 Verify scenario "Contributor guide names the fresh-build replay check" with `git diff --check` and full Octet verification; save output at `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/final-verification.txt`.
- [x] V4 Run full Octet verification; save output at `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/final-verification.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L`; input=`cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json`; asserts=`fresh Nix-built binaries regenerate matching expanded corpus evidence without correctness or semantic-equivalence claims`; evidence=`openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/fresh-build-object-corpus-check.txt`

## Change Status

- [x] Complete

## Verification Coverage

- `Scenario: Fresh-build binaries reproduce expanded corpus receipt` -> `V1` -> `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/fresh-build-object-corpus-check.txt`
- `Scenario: Fresh-build rail preserves bounded non-proof claims` -> `V2` -> `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/fresh-build-object-corpus-check.txt`
- `Scenario: Contributor guide names the fresh-build replay check` -> `V3` -> `openspec/changes/archive/2026-05-19-fresh-build-object-corpus-determinism/evidence/final-verification.txt`
