## Why

Octet now has a first-class Verus proof-rail path, but the integration is still fixture-grade. The Nix `octet-verus` surface is a deterministic wrapper that greps proof files instead of invoking real Verus, proof receipts are line-oriented text, `cargo octet prove` executes shell command strings, and source/proof rail matching accepts substring matches. Those choices were useful for proving the evidence pipeline, but they are too weak for promotion-grade formal-proof evidence.

The next improvement should harden the rail without changing the starter lint path. Verus remains opt-in and scoped, while proof evidence becomes structured, reproducible, and safer to execute and verify.

## What Changes

- Replace fixture-only verifier identity with a real-Verus-ready pinned verifier surface that records Verus, solver, Rust toolchain, and Nix closure identity; keep fixture execution only as an explicitly marked fast-test mode.
- Replace line-substring receipt validation with a Rust-owned structured Verus proof receipt and generated Nickel contract freshness checks.
- Change proof execution from shell strings to argv-based commands with deterministic working directory, environment allowlist, configured timeout, bounded logs, and fail-closed diagnostics.
- Replace substring source/proof matching with normalized repo-relative exact and glob matching that rejects ambiguous or overbroad bindings.
- Add a Nickel-authored proof-rail manifest for human-owned rail declarations, exported to deterministic JSON for `cargo octet prove` and standards validation.
- Improve operator ergonomics with list/explain surfaces for selected rails, unbacked verified claims, and stale receipt fields.

## Non-Goals

- Do not require Verus for ordinary `cargo octet check` starter usage.
- Do not claim whole-program correctness, semantic equivalence, certification, business-domain validation, or full safety.
- Do not make Dylint lints depend on Verus execution.
- Do not upload proofs, source, logs, or receipts to hosted services.
- Do not accept ambient `verus` or solver binaries for `formal_proof` provenance.
- Do not attempt broad Octet semantic verification beyond a small promotion-grade rail fixture in this change.

## Impact

- **Files**: `flake.nix`, `cargo-octet/src/prove.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/src/evidence*.rs`, `crates/octet-standards/src/proof.rs`, generated contract artifacts, `standards/policies/registry.ncl`, `verus/`, docs, fixtures, and tests.
- **APIs**: `cargo octet prove` gains structured receipt output and safer rail selection; optional list/explain flags become review aids.
- **Config**: proof rail declarations move toward Nickel-authored contracts with deterministic JSON export where humans maintain rail config.
- **Dependencies**: a pinned real Verus/Z3 surface is introduced when available through Nix; fixture verifier remains isolated from production `formal_proof` evidence.
- **Testing**: positive and negative tests cover structured receipts, stale bindings, verifier mismatch, shell-free execution, path matching, manifest validation, and starter-path non-requirement.
