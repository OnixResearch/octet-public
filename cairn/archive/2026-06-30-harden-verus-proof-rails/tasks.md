## Contract

- [x] [serial] Specify production Verus verifier identity requirements and fixture-only verifier boundaries. r[octet.verus_proof_rails.verifier_identity]
- [x] [serial] Specify structured Verus proof receipts, Rust-derived contract ownership, and fail-closed receipt validation. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Specify shell-free proof execution with deterministic environment, configured timeout, and bounded logs. r[octet.verus_proof_rails.safe_execution]
- [x] [serial] Specify normalized exact/glob source and proof path matching. r[octet.verus_proof_rails.path_matching]
- [x] [serial] Specify Nickel-authored proof rail manifests and deterministic JSON export. r[octet.verus_proof_rails.rail_manifest]
- [x] [serial] Specify operator list/explain behavior for selected rails and unbacked proof claims. r[octet.verus_proof_rails.operator_feedback]

## Implementation

- [x] [serial] Run current focused Verus proof-rail tests before core changes and record any baseline failures. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Add a Rust-owned `VerusProofReceipt` DTO plus pure constructors, validators, and deterministic diagnostics. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Generate and commit the Nickel contract for Verus proof receipts, then register the receipt family in `standards/policies/registry.ncl`. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Update `cargo octet prove` to write structured receipt JSON plus bounded stdout/stderr artifacts instead of relying on text-line receipt membership. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Replace proof command shell strings with executable-plus-args command data, pinned verifier path resolution, environment allowlist, configured timeout, and typed command results. r[octet.verus_proof_rails.safe_execution]
- [x] [serial] Add normalized repo-relative exact/glob matching for rail source paths, proof paths, verified-module coverage, and proof artifact verification. r[octet.verus_proof_rails.path_matching]
- [x] [serial] Split fixture verifier identity from production Verus identity, including explicit fixture-only provenance labels and production rejection rules. r[octet.verus_proof_rails.verifier_identity]
- [x] [serial] Add the Nickel proof-rail manifest contract and deterministic JSON export path for human-authored rail declarations. r[octet.verus_proof_rails.rail_manifest]
- [x] [serial] Add `cargo octet prove --list` and explain diagnostics for unmatched rails, stale fields, verifier mismatch, and unbacked verified claims. r[octet.verus_proof_rails.operator_feedback]
- [x] [serial] Update `verus/README.md`, main README usage, and consumer guidance to describe scoped structured receipts, fixture-only boundaries, and starter-path non-requirement. r[octet.verus_proof_rails.operator_feedback]

## Verification

- [x] [serial] Add positive tests for a valid structured receipt with matching verifier identity, pass signal, source binding, proof binding, trust boundaries, and `formal_proof` provenance. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Add negative tests for malformed receipt JSON, missing required fields, stale source hash, stale proof hash, wrong verifier identity, missing proof artifact identity, and fixture receipt promoted as production proof. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Add positive and negative tests for shell-free command construction, spawn failure, non-zero verifier exit, configured timeout, and bounded stdout/stderr artifacts. r[octet.verus_proof_rails.safe_execution]
- [x] [serial] Add positive and negative tests for exact path matches, glob matches, empty patterns, ambiguous patterns, and rejected substring-only matches. r[octet.verus_proof_rails.path_matching]
- [x] [serial] Add positive and negative Nickel manifest fixtures for valid rails, missing verifier identity, invalid backend, empty source paths, and malformed trust boundaries. r[octet.verus_proof_rails.rail_manifest]
- [x] [serial] Add CLI parse/report tests for `cargo octet prove --list` and explain diagnostics. r[octet.verus_proof_rails.operator_feedback]
- [x] [serial] Rerun focused `cargo-octet` and `octet-standards` tests after core changes. r[octet.verus_proof_rails.structured_receipts]
- [x] [serial] Run `nix run .#cairn -- validate --root .` and Cairn proposal/design/tasks gates for this change. r[octet.verus_proof_rails.rail_manifest]
- [x] [serial] Run a starter-path regression showing `cargo octet check` does not require Verus or proof manifests. r[octet.verus_proof_rails.safe_execution]
