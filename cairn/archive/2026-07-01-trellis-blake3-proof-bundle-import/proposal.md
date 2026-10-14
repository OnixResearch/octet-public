## Why

Trellis has a mature Verus proof corpus, BLAKE3 receipt/hash policy, proof-review automation, and explicit BLAKE3 trust-boundary modeling. Octet can use Trellis as a reference corpus for proof-card and proof-path design, but it must not silently upgrade external hashes into Octet formal-proof evidence.

A first-class imported-proof bundle contract lets Octet reference Trellis proof artifacts by BLAKE3 identity, show those references in proof cards, and optionally rerun selected proofs through Octet's pinned Verus proof rail before emitting `formal_proof` provenance.

## What Changes

- Define a Trellis imported-proof bundle shape with repository revision, source path, proof path, command, verifier identity if known, bounded stdout/stderr hashes, receipt hash when present, and trust-boundary notes.
- Author the human-maintained import manifest as typed Nickel and export deterministic JSON for runtime Rust consumers.
- Register any generated import bundle and receipt family in the policy registry as Nickel-authored, Rust-derived, or non-candidate with rationale.
- Treat imported Trellis bundles as external/manual-review evidence until Octet reruns the proof with a pinned verifier and validates an Octet-owned receipt.
- Add positive and negative fixtures for exact BLAKE3 identity checks, stale hash rejection, missing field rejection, and imported-as-`formal_proof` rejection.

## Non-Goals

- Do not vendor Trellis source or make Octet depend on Trellis at runtime.
- Do not claim Trellis proof correctness from BLAKE3 hashes alone.
- Do not broaden Octet's production Verus scope or require Trellis for ordinary `cargo octet check`.
- Do not replace Octet structured receipts, provenance, or artifact verification.
- Do not import Trellis trusted hash axioms as Octet proof obligations without a separate verifier-scoped change.

## Impact

- **Files**: import manifest/contracts under standards or proof-rail policy paths, generated JSON fixture, proof-card/import validation helpers, `verus/README.md`, `docs/trust-boundary-evidence.md`, and README references.
- **Behavior**: imported bundles become reviewable inputs and card references; only Octet reruns with validated receipts can emit Octet `formal_proof` provenance.
- **Testing**: positive import fixture, negative stale/malformed/overclaim fixtures, policy freshness checks, artifact verifier integration as needed, and Cairn validation/gates.
