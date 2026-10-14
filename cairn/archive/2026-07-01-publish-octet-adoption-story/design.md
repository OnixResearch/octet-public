## Context

Octet's README already explains the checked path, quick start, proof rails, release/operator smoke path, and scope boundaries. Those pieces are accurate but distributed. A first-time reader comparing Octet to a polished verified-systems-language site should immediately understand that Octet's advantage is incremental, auditable Rust adoption: existing Rust source enters `cargo octet check`, policy/standards checks, optional Verus proof rails, and artifact verification without a rewrite.

The adoption story should therefore be a docs/evidence packaging change. It should reuse existing Octet facts, archived downstream evidence, and scope-boundary wording instead of introducing unsupported product claims.

## Approach

1. **Canonical adoption page or section.** Add one canonical adoption-story surface and link to it from the README quick-start or documentation list. The surface should start with the goal: evidence-backed Rust systems code checked before release.
2. **Built-with case-study contract.** Define a repeatable case-study shape: consumer repository or archive reference, checked scope, command, configuration, evidence artifact path, findings or suppressions summary, rollout status, and out-of-scope claims.
3. **Aspen seed case.** If Aspen is used, cite `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical phase-1 starter-rollout reference, not an arbitrary local Aspen checkout.
4. **Methodology before metrics.** Any performance, cache, CI-latency, false-positive, or suppression-rate claim should include command, input scope, environment, raw artifact or transcript, and interpretation boundary.
5. **Boundary-preserving comparison.** The docs may contrast Octet's Rust-compatible evidence path with new-language approaches, but must not name unsupported guarantees or imply Octet proves all code correct.
6. **Deterministic docs checks.** Add or reuse a focused checker that inventories required headings/fields and rejects high-risk overclaims.

## Decisions

### 1. Position Octet as an adoption path, not a language

**Choice:** The primary message should be "evidence-backed Rust systems code, checked before release" or equivalent wording.

**Rationale:** Octet's strongest differentiator is compatibility with Rust workspaces and release gates. Copying a new-language promise would be inaccurate and less useful to existing Rust teams.

### 2. Case studies require reviewable evidence

**Choice:** A "Built with Octet" entry is acceptable only when it names the checked scope, command, config, evidence artifact or archive path, and scope boundary.

**Rationale:** A case-study list without evidence would dilute Octet's core trust model. The reader should be able to replay or inspect what was checked.

### 3. Metrics are methodology-gated

**Choice:** Public performance/adoption metrics should be published only with raw commands and artifacts, not as standalone claims.

**Rationale:** Salt's benchmark packaging is effective because it states methodology. Octet should match that discipline for lint overhead, cache behavior, CI latency, and rollout cost.

### 4. Negative wording is part of acceptance

**Choice:** Documentation validation should reject known high-risk unsupported claims.

**Rationale:** The purpose is clearer positioning without marketing drift. A forbidden-claim check keeps the change aligned with Octet's existing scope-boundary contract.

## Risks / Trade-offs

- **Evidence availability:** A polished case-study page may expose gaps in current public artifacts. Mitigate by starting with archived evidence and marking incomplete candidates as not yet published.
- **Docs duplication:** README, consumer guide, trust-boundary docs, and downstream rollout docs could diverge. Mitigate with one canonical adoption page and short cross-links.
- **Overcautious copy:** Boundary wording could weaken the hero message. Mitigate by pairing every non-claim with concrete supported evidence surfaces.
- **Brittle wording checks:** Exact phrase checks can cause churn. Mitigate by checking a small set of required fields and high-risk forbidden phrases instead of general words like "proof".

## Validation

- Positive documentation inventory confirms the canonical adoption story contains the Rust-compatible promise, checked-path stages, case-study field list, methodology field list, and at least one evidence-backed case-study reference or explicit placeholder policy.
- Positive Aspen-specific check confirms any Aspen case study cites `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` when Aspen phase-1 evidence is used.
- Negative wording check rejects claims that Octet is a new language, replaces Rust, guarantees mathematical certainty, proves whole-program correctness, certifies systems, or has a verified runtime without separate proof.
- Cairn `validate` and proposal/design/tasks gates pass for this change package.
