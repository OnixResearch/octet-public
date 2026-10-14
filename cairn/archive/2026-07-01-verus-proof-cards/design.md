## Context

Octet's Verus proof rails already persist structured receipts and provenance. Reviewers can verify those artifacts, but the review entry point is still detailed and command-oriented. external evidence tooling's proof cards are useful because they compress each claim into a named, scoped summary. Octet can adopt that shape if the card remains subordinate to the receipt and evidence bundle.

This change depends on the Verus proof-path explainer so readers understand where cards fit in the route from requirement to artifact verification.

## Approach

1. **Define a proof-card field set.** Required fields: proof/property ID, source requirement or policy ID, checked Rust source scope, proof target path, verifier mode, verifier identity summary, solver/toolchain summary, receipt path, artifact BLAKE3 identities, evidence tier, status, trust boundaries, and non-claims.
2. **Use receipt-backed content.** Prefer generating cards from structured receipt/provenance data. If cards are written as docs first, add deterministic checks that compare card fields to receipt or manifest fields.
3. **Make cards indexes, not evidence replacements.** A card may summarize status only when it names the receipt/provenance artifact and artifact-verifier command. Without a matching receipt, it must say `manual_review_required` or `not executed` rather than `formal_proof`.
4. **Support imported Trellis references as a separate card mode.** Trellis-backed cards may name source, proof, stdout, stderr, and receipt BLAKE3 identities plus the Trellis revision. They remain external/manual-review evidence unless Octet reruns the proof with a pinned verifier and validates an Octet-owned receipt.
5. **Cover fixture, production, and external modes.** Fixture-only rails must be visibly labeled as fixture-only. Production pilots may show `formal_proof` only when the matching validated receipt supports it. External Trellis cards must show that BLAKE3 identifies bytes, not correctness.
6. **Add negative fixtures.** Reject cards that omit required fields, point at stale receipts, confuse fixture/production/external mode, or contain unsupported whole-program proof wording.

## Decisions

### 1. Cards are derived summaries

**Choice:** Treat proof cards as receipt-backed summaries, not a separate source of truth.

**Rationale:** The authoritative evidence remains the structured receipt, provenance record, and artifact-verifier result.

### 2. Missing receipt is not proof

**Choice:** A card without a matching validated receipt cannot claim `formal_proof`.

**Rationale:** This keeps the card surface aligned with Octet's evidence-tier taxonomy and prevents stale docs from becoming evidence.

### 3. Same card shape for fixture, production, and external references

**Choice:** Use one card shape with required verifier-mode and evidence-origin fields.

**Rationale:** Reviewers should be able to compare fixture plumbing, production pilots, and imported Trellis proof references without confusing their promotion status.

## Risks / Trade-offs

- **Stale documentation:** Hand-written cards can drift. Mitigate with generated cards or receipt/manifest comparison checks.
- **External proof confusion:** Imported Trellis hashes can look stronger than they are. Mitigate by requiring an external/manual-review tier until Octet reruns and validates the proof.
- **Overclaim through compact wording:** Cards can hide assumptions. Mitigate with required trust-boundary and non-claim fields.
- **Artifact sprawl:** Adding another artifact may burden operators. Mitigate by making cards optional review summaries that point to existing receipts.

## Validation

- Positive card inventory covers current Verus fixture rail, production pilot rails, and any imported Trellis card fixtures, with all required fields present.
- Positive receipt linkage check proves each `formal_proof` card points to a matching validated receipt/provenance artifact or explicitly marks itself as not executed.
- Positive external-reference check proves Trellis cards expose repository revision plus BLAKE3 source/proof/log/receipt identities while staying non-`formal_proof` unless rerun by Octet.
- Negative checks reject missing field, stale receipt, fixture-as-production, external-as-formal-proof, and unsupported broad correctness claim cases.
- Cairn validation and gates pass for this change package before implementation and before archive.
