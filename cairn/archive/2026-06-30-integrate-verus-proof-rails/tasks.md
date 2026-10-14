## Contract

- [x] [serial] Define the Verus proof-rail contract, pinned verifier identity requirements, and scoped non-claim wording. r[octet.verus_proof_rails.contract]
- [x] [serial] Define persistent proof receipt and provenance requirements for `formal_proof` artifacts. r[octet.verus_proof_rails.receipts]
- [x] [serial] Define verified-module proof coverage behavior for selected verification profiles. r[octet.verus_proof_rails.verified_module_coverage]

## Implementation

- [x] [serial] Add the pinned Verus verifier wrapper or Nix package surface and deterministic verifier identity output. r[octet.verus_proof_rails.contract]
- [x] [serial] Extend `octet-standards` proof-rail execution so the CLI shell can persist receipts while the pure core validates pass signals, BLAKE3 bindings, provenance lines, and verifier metadata from in-memory inputs. r[octet.verus_proof_rails.receipts]
- [x] [serial] Add `cargo octet prove` as a thin shell over selected standards verification rails, with artifact-dir output and stable human/JSON reporting. r[octet.verus_proof_rails.prove_command]
- [x] [serial] Emit `formal_proof` provenance only when verifier identity and proof artifact identity are present, and make `cargo octet artifact verify` cover proof receipts without mutating artifact directories. r[octet.verus_proof_rails.receipts]
- [x] [serial] Register Verus proof receipt and verifier identity families in `standards/policies/registry.ncl` as Rust-derived or explicit non-candidates with rationale. r[octet.verus_proof_rails.registry]
- [x] [serial] Replace the placeholder Verus example with one small Octet-owned proof target and declared proof rail. r[octet.verus_proof_rails.contract]
- [x] [serial] Add docs for `cargo octet prove`, scoped Verus proof claims, and starter-profile non-requirement. r[octet.verus_proof_rails.contract]

## Verification

- [x] [serial] Add positive tests proving a pinned Verus fixture receipt validates with matching pass signal, source BLAKE3 binding, verifier identity, and `formal_proof` provenance. r[octet.verus_proof_rails.receipts]
- [x] [serial] Add negative tests for missing proof artifact, missing pass signal, stale source binding, wrong verifier identity, unknown proof backend, and `formal_proof` provenance without proof artifact identity. r[octet.verus_proof_rails.receipts]
- [x] [serial] Add positive and negative verified-module coverage fixtures: covered `verified/` module passes under verification profile, unbacked claim emits or fails as `manual_review_required` according to profile. r[octet.verus_proof_rails.verified_module_coverage]
- [x] [serial] Run focused `cargo-octet` CLI parse/report tests for `prove`. r[octet.verus_proof_rails.prove_command]
- [x] [serial] Run focused `octet-standards` proof-rail tests and artifact verifier tests. r[octet.verus_proof_rails.receipts]
- [x] [serial] Run `cairn validate --root .` and Cairn gates for this change. r[octet.verus_proof_rails.contract]
