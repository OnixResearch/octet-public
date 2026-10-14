## Context

Octet's accepted Verus proof-rail spec already requires scoped receipts, production verifier identity, fail-closed validation, structured provenance, and starter-path independence. The missing piece is a concise reader-facing route that explains how those pieces compose into one formal-proof evidence lane.

The explainer should adapt the useful external evidence tooling shape without copying its language-core or numerical-domain claims. Octet's route is not `requirement -> external reference property -> SMT proof`; it is `requirement or policy ID -> Verus rail -> source/proof binding -> verifier run -> structured receipt -> formal_proof provenance -> artifact verifier`.

## Approach

1. **Add one canonical proof-path paragraph or diagram.** Put the route where operators already look first: README quick-start formal-proof section, `docs/trust-boundary-evidence.md`, or `verus/README.md`.
2. **Name each stage in Octet terms.** Use `requirement or policy ID`, `Verus proof rail`, `checked Rust source scope`, `proof target`, `verifier/solver/toolchain identity`, `structured receipt`, `formal_proof provenance`, and `artifact verifier`.
3. **Link commands to stages.** Tie selection/listing to `cargo octet prove --list`, explanation to `cargo octet prove --explain`, execution to `cargo octet prove`, and bundle checking to `cargo octet artifact verify`.
4. **Keep tier separation explicit.** Say that Verus-backed receipts may support `formal_proof`; lint findings remain `static_lint`, policies remain `policy_check`, transcripts remain `transcript_assertion`, and unresolved claims remain `manual_review_required`.
5. **Add deterministic wording checks.** Validate that required proof-path stage labels are present and unsupported unqualified claims are absent.

## Decisions

### 1. Documentation only

**Choice:** This change should not alter proof execution or receipt schema.

**Rationale:** The accepted Verus proof-rail spec already owns those contracts. This change makes them inspectable and teachable.

### 2. Proof path is a lane, not the foundation

**Choice:** Describe Verus as one high-assurance evidence lane inside the Octet trust stack.

**Rationale:** Octet's foundation remains Rust-compatible static analysis, policy checks, and evidence provenance. Verus proofs are scoped artifacts for declared properties, not a whole-language soundness theorem.

### 3. Negative wording is part of acceptance

**Choice:** Treat non-claim wording as a deterministic validation surface.

**Rationale:** A proof-path explainer can accidentally broaden claims. The change is only acceptable if it preserves explicit boundaries.

## Risks / Trade-offs

- **Marketing drift:** The explainer could imply blanket correctness. Mitigate with forbidden-claim checks and explicit non-goals.
- **Duplication:** README, trust-boundary docs, and `verus/README.md` could diverge. Mitigate with one canonical route and shorter links from other docs.
- **Terminology confusion:** Readers may equate validation with proof. Mitigate by naming evidence tiers at the route boundary.

## Validation

- Positive documentation check shows the proof path includes requirement/policy ID, Verus rail, source/proof binding, verifier identity, structured receipt, `formal_proof` provenance, and artifact verification.
- Positive command inventory shows `cargo octet prove --list`, `cargo octet prove --explain`, `cargo octet prove`, and `cargo octet artifact verify` are discoverable from the explainer.
- Negative wording check rejects unqualified whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or generic verified-runtime claims.
- Cairn validation and gates pass for the change package before implementation and before archive.
