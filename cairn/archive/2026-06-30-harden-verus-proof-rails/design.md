## Context

The archived `integrate-verus-proof-rails` change established the product boundary: Verus is an opt-in standards proof rail, not a lint, and `formal_proof` evidence must name verifier identity, proof artifact identity, checked scope, and trust boundaries. The remaining weaknesses are implementation hardening issues:

- `octet-verus` currently behaves as a fixture verifier rather than a real Verus wrapper.
- Receipts are text blobs validated by required-line membership.
- Proof commands are configured as shell strings.
- Rail/source matching uses substring checks.
- Human-authored proof rail configuration does not yet have a typed Nickel surface.

This change keeps the same product model and strengthens the evidence path.

## Approach

1. **Separate verifier modes.** Production `formal_proof` evidence must use the pinned Verus wrapper identity. Fixture-only verifier output may remain available for fast tests, but receipts from fixture mode must be marked as fixture scope and must not be promoted as production proof evidence.
2. **Introduce a structured receipt DTO.** Add a Rust-owned `VerusProofReceipt` model that records schema version, policy ID, rail index, verifier identity, command identity, source bindings, proof bindings, trust boundaries, pass/fail status, bounded log artifact identities, and non-claim text. Generate a Nickel contract from the Rust schema and register it as a Rust-derived receipt family.
3. **Keep validation pure.** Receipt validation, binding comparison, verifier identity comparison, path-pattern matching, and diagnostic construction live in deterministic pure functions over in-memory values. CLI code owns filesystem reads/writes, process execution, environment construction, and JSON rendering.
4. **Replace shell execution.** Model proof commands as executable plus args. The imperative shell resolves the pinned verifier path, sets an allowlisted environment, applies the configured timeout, captures bounded stdout/stderr artifacts, and returns a typed command result to the core.
5. **Normalize source matching.** Convert source paths to repo-relative normalized paths before matching. Exact patterns match one path; glob patterns match one or more paths; empty, ambiguous, and overbroad patterns emit deterministic diagnostics. Plain substring matching is removed from proof coverage decisions.
6. **Add a Nickel proof-rail manifest.** Human-authored rail declarations use Nickel contracts and export deterministic JSON consumed by existing registry/prove paths. Runtime Rust consumes the exported JSON, not Nickel directly.
7. **Improve operator feedback.** `cargo octet prove --list` and explain-style diagnostics should show selected rails, unmatched verified claims, stale source hashes, verifier mismatches, and the exact remediation without implying broader proof claims.

## Data Model Sketch

`VerusProofReceipt` should be compact and deterministic:

- `schema_version`
- `policy_id`
- `rail_index`
- `backend`
- `verifier_identity`
- `toolchain_identity`
- `command_blake3`
- `source_bindings`
- `proof_bindings`
- `trust_boundaries`
- `checked_scope`
- `status`
- `pass_signal`
- `stdout_artifact`
- `stderr_artifact`
- `non_claims`

BLAKE3 remains the default content hash for command, source, proof, and artifact identities.

## Key Decisions

### Structured receipts replace line-oriented trust

Line-oriented receipts made the first integration cheap, but they allow accidental acceptance when unrelated log text happens to contain an expected line. A typed receipt lets Octet reject malformed, stale, missing, and mismatched fields deterministically.

### Fixture verification is not production proof

The fixture wrapper is still valuable for unit tests and local proof-plumbing checks. It must be labeled as fixture evidence and kept out of promotion-grade `formal_proof` provenance unless the checked scope explicitly says it is fixture-only.

### Proof rail config is human-authored Nickel, runtime JSON

Nickel gives maintainers contracts, merge semantics, and type errors before execution. The runtime path stays Rust-only by consuming checked-in exported JSON.

### Path matching must be exact enough to audit

A proof receipt is only useful if reviewers can tell exactly which source and proof artifacts were bound. Ambiguous glob matches should be visible, and substring matching should not silently bind `foo/verified/bar.rs` to unrelated paths.

## Risks

- **Real Verus packaging cost**: Verus/Z3 pinning may take longer than receipt hardening. Mitigate by keeping the production wrapper contract explicit and allowing fixture-only tests to cover core behavior until the Nix package is available.
- **Receipt migration churn**: existing tests and docs expect line receipts. Mitigate with a short transition path where legacy fixture receipts are accepted only in fixture mode.
- **Config duplication**: registry and manifest fields may drift. Mitigate by exporting deterministic JSON and checking freshness in tests.
- **Log over-capture**: verifier output can include environment-specific details. Mitigate with bounded log artifacts, digests in receipts, and explicit non-candidate rationale for raw internals.

## Validation

- Positive: real-or-fixture pinned verifier run writes a valid structured receipt, provenance record, bounded logs, source/proof bindings, and artifact verification result.
- Negative: malformed receipt JSON, missing pass signal, stale source hash, stale proof hash, wrong verifier identity, fixture receipt promoted as production, ambiguous path match, unbacked verified claim, command spawn failure, timeout, and invalid manifest all fail closed.
- Regression: `cargo octet check` starter path does not require Verus, proof artifacts, or proof manifests.
