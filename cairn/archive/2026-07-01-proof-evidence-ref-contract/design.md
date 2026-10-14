## Context

`cargo octet prove` emits `proof-cards.json` as a human review index backed by structured receipts and provenance. The next integration step is allowing other review surfaces to point at a proof card in a machine-checkable way. The reference must remain data because proof evidence is external to Rust type checking and depends on artifact validation.

## Approach

1. **DTO, not trait.** Add `ProofEvidenceRef` as a serde DTO with schema version `octet-proof-evidence-ref/v1`.
2. **Read-only validator.** Add `cargo octet proof-ref validate --proof-cards PATH --ref PATH [--output-format human|json]` that reads both JSON files, validates linkage in pure core, and exits non-zero on invalid references.
3. **Exact card matching.** Match by `proof_or_property_id` first, then require source requirement/policy ID, checked scope, evidence tier, and status to match the reference.
4. **Artifact and non-claim coverage.** Require every `required_artifacts` entry in the reference to appear in the card artifact identities, and every reference non-claim to appear in the card non-claims.
5. **No proof promotion.** The command validates linkage only. It does not rerun Verus, does not emit `formal_proof` provenance, and does not make manual-review cards formal proof.

## Decisions

### 1. Use proof cards as the target

**Choice:** The DTO points at `proof-cards.json`, not directly at receipts.

**Rationale:** Proof cards already normalize fixture/production/external modes, non-claims, artifacts, and verifier summaries for reviewers. Receipt/provenance validation remains owned by `cargo octet prove` and `cargo octet artifact verify`.

### 2. Keep required artifacts path-only in v1

**Choice:** A reference names required artifact locators and relies on the card for BLAKE3 identities.

**Rationale:** This makes source comments and suppression records small. If a future consumer needs digest pinning, v2 can add expected BLAKE3 fields.

### 3. Formal proof remains explicit

**Choice:** A reference that expects `formal_proof` must match a card that already reports `formal_proof`; the validator cannot upgrade a manual-review card.

**Rationale:** This preserves Octet's evidence-tier boundaries.

## Risks / Trade-offs

- **Reference drift:** Cards change when proof rails move. Mitigate with deterministic validation diagnostics naming mismatched fields.
- **False confidence:** A valid reference might be read as proof by itself. Mitigate with required non-claims and help text stating the command validates linkage only.
- **Schema churn:** Future suppression integration may need more fields. Mitigate with a schema version and small v1 field set.

## Validation

- Positive unit test validates a reference against a matching formal-proof card.
- Negative unit tests cover missing card, mismatched scope, mismatched evidence tier/status, missing required artifact, and missing non-claim.
- CLI parse/help tests cover the new subcommand shape.
- Focused `cargo test -p cargo-octet proof_ref` passes after implementation.
- Cairn `validate` and proposal/design/tasks gates pass.
