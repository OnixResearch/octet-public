## Context

Octet's accepted Verus proof-rail spec already requires scoped receipts, production verifier identity, fail-closed validation, structured provenance, and starter-path independence. The missing piece is a concise reader-facing route that explains how those pieces compose into one formal-proof evidence lane.

The explainer should adapt the useful external evidence tooling shape without copying its language-core or numerical-domain claims. Octet's direct route is `requirement or policy ID -> Verus rail -> source/proof binding -> verifier run -> structured receipt -> formal_proof provenance -> artifact verifier`.

Trellis adds a second useful route for external proof references. It has large Verus proof coverage, BLAKE3 receipt/hash policy, proof-review automation, and BLAKE3-modeled trusted hash boundaries such as `src/chain.rs`, `src/snapshot_integrity.rs`, and `src/ucan_proof_reference.rs`. Octet can reference those artifacts by content identity, but the explainer must keep imported BLAKE3 references separate from Octet-owned formal proof evidence.

## Approach

1. **Add one canonical proof-path paragraph or diagram.** Put the route where operators already look first: README quick-start formal-proof section, `docs/trust-boundary-evidence.md`, or `verus/README.md`.
2. **Name each direct stage in Octet terms.** Use `requirement or policy ID`, `Verus proof rail`, `checked Rust source scope`, `proof target`, `verifier/solver/toolchain identity`, `structured receipt`, `formal_proof provenance`, and `artifact verifier`.
3. **Add an imported Trellis branch.** Show `Trellis revision -> source/proof/log/receipt BLAKE3 identities -> imported proof bundle -> proof card/manual review -> optional Octet rerun`. This branch is external evidence until Octet runs the proof with pinned verifier identity and validates an Octet-owned receipt.
4. **Link commands to stages.** Tie selection/listing to `cargo octet prove --list`, explanation to `cargo octet prove --explain`, execution to `cargo octet prove`, and bundle checking to `cargo octet artifact verify`.
5. **Keep tier separation explicit.** Say that Verus-backed receipts may support `formal_proof`; lint findings remain `static_lint`, policies remain `policy_check`, transcripts remain `transcript_assertion`, imported Trellis bundles remain external/manual-review references until rerun, and unresolved claims remain `manual_review_required`.
6. **Add deterministic wording checks.** Validate that required proof-path stage labels are present and unsupported unqualified claims are absent.

## Decisions

### 1. Documentation only

**Choice:** This change should not alter proof execution or receipt schema.

**Rationale:** The accepted Verus proof-rail spec already owns those contracts. This change makes them inspectable and teachable.

### 2. Proof path is a lane, not the foundation

**Choice:** Describe Verus as one high-assurance evidence lane inside the Octet trust stack.

**Rationale:** Octet's foundation remains Rust-compatible static analysis, policy checks, and evidence provenance. Verus proofs are scoped artifacts for declared properties, not a whole-language soundness theorem.

### 3. Trellis imports are content-addressed references

**Choice:** Use Trellis BLAKE3 proof artifacts as imported references and optional rerun inputs, not as automatic Octet formal-proof evidence.

**Rationale:** BLAKE3 binds exact bytes. It does not establish that the proof ran under Octet's verifier identity, source scope, receipt schema, or artifact verifier.

### 4. Negative wording is part of acceptance

**Choice:** Treat non-claim wording as a deterministic validation surface.

**Rationale:** A proof-path explainer can accidentally broaden claims. The change is only acceptable if it preserves explicit boundaries.

## Risks / Trade-offs

- **Marketing drift:** The explainer could imply blanket correctness. Mitigate with forbidden-claim checks and explicit non-goals.
- **Imported evidence drift:** Trellis artifact hashes can go stale or be mistaken for proof correctness. Mitigate by requiring repository revision and hash-field checks, plus non-`formal_proof` wording until Octet reruns.
- **Duplication:** README, trust-boundary docs, and `verus/README.md` could diverge. Mitigate with one canonical route and shorter links from other docs.
- **Terminology confusion:** Readers may equate validation with proof. Mitigate by naming evidence tiers at the route boundary.

## Validation

- Positive documentation check shows the proof path includes requirement/policy ID, Verus rail, source/proof binding, verifier identity, structured receipt, `formal_proof` provenance, and artifact verification.
- Positive imported-branch check shows the proof path includes Trellis revision, BLAKE3 source/proof/log/receipt identities, imported bundle status, and optional Octet rerun.
- Positive command inventory shows `cargo octet prove --list`, `cargo octet prove --explain`, `cargo octet prove`, and `cargo octet artifact verify` are discoverable from the explainer.
- Negative wording check rejects unqualified whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, generic verified-runtime claims, and imported Trellis references labeled as Octet `formal_proof` without rerun evidence.
- Cairn validation and gates pass for the change package before implementation and before archive.
