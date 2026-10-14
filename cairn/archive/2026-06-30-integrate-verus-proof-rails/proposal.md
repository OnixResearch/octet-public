## Why

Octet already has the right evidence framing for formal proof, but Verus is still only a placeholder under `verus/` and is not part of any enforced checked path. The result is a gap: `verified_purity` can keep `verified/` modules free of async, IO, and wall-clock effects, while Octet cannot yet prove or reject whether those modules have a pinned, replayable Verus proof rail.

Verus should enter Octet as a first-class, opt-in formal-proof rail with explicit verifier identity, source/proof binding, persistent receipts, and artifact verification. It must not be treated as another lint or as a blanket correctness claim.

## What Changes

- Add a pinned Verus verifier surface through Nix and an Octet-owned wrapper command for selected proof rails.
- Extend the existing `ProofRail` execution path so Verus receipts can be persisted under an artifact directory and verified later instead of existing only as in-memory check input.
- Add a `cargo octet prove` command that runs selected proof rails, emits `formal_proof` provenance, and preserves the existing distinction between lint, policy, transcript, manual-review, and formal-proof evidence tiers.
- Register Verus proof receipts and verifier identity material in `standards/policies/registry.ncl` as Rust-derived receipt families or explicit non-candidates where raw verifier internals do not belong in Nickel.
- Require modules or docs that make `verified`/formal-proof claims to map to a declared proof rail, or else emit/fail with `manual_review_required` under the selected verification profile.
- Replace the placeholder Verus example with a small real Octet-owned proof target plus positive and negative fixtures for stale source bindings, missing pass signals, wrong verifier identity, missing proof artifacts, and unbacked verified-module claims.

## Non-Goals

- Do not claim whole-program correctness, semantic equivalence, certification, or business-domain validation.
- Do not require Verus for ordinary `cargo octet check` starter usage.
- Do not upload source, proofs, or receipts to any hosted service.
- Do not make Octet's Dylint lints depend on Verus execution.
- Do not trust an ambient `verus` binary from `PATH` when producing formal-proof evidence.

## Impact

- **Files**: `flake.nix`, `cargo-octet/src/`, `crates/octet-standards/src/`, `standards/policies/registry.ncl`, generated policy artifacts, `verus/` or `proofs/verus/`, docs, fixtures, and tests.
- **APIs**: new `cargo octet prove` command and proof-artifact options; standards proof-rail execution gains persistent receipt output.
- **Dependencies**: pinned Verus verifier package or wrapper through the repo flake; no ambient verifier dependency for formal proof receipts.
- **Testing**: focused Rust tests for CLI parsing, receipt validation, artifact verification, and standards proof rails; positive and negative proof fixtures; Nix check for the pinned Verus proof target when available.
