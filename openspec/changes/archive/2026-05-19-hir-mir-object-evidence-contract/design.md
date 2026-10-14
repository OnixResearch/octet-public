## Context

Function Object v1 currently uses source-token evidence, deterministic JSON receipts, BLAKE3 object/set hashes, explicit dependency/effect/macro caveats, compatibility fixtures, internal corpus gates, fresh-target repeatability, and fresh-build receipt parity. Those rails intentionally do not resolve Rust semantics through rustc HIR or MIR.

## Goals / Non-Goals

**Goals:**

- Define HIR/MIR evidence as additive supplements to Function Object v1.
- Require rustc/toolchain scoping and extraction caveats for every compiler-derived evidence family.
- Preserve non-proof boundaries: no correctness claim, no semantic-equivalence claim, no cross-toolchain stability claim.
- Create a spec target for later implementation work: DTOs, contract metadata, fixtures, and deterministic gates.

**Non-Goals:**

- Implement HIR or MIR extraction in this scaffold.
- Replace source-token Function Object v1 hashes.
- Promise stable HIR/MIR identity across rustc versions, targets, features, or optimization contexts.
- Turn Valence object hashes into callable code, whole-workspace content addressing, or correctness proofs.

## Decisions

### 1. Supplements, not replacements

**Choice:** HIR/MIR evidence will be modeled as optional object supplements attached to existing Function Object v1 identity.

**Rationale:** Existing receipts and corpus gates already provide deterministic review/provenance identity. HIR/MIR can improve evidence quality, but compiler-derived structures are toolchain/context-sensitive.

**Alternative:** Replace source-token object identity with HIR/MIR hashes. Rejected because it would destabilize current v1 contracts and overstate compiler-version portability.

### 2. Fail-closed support declaration

**Choice:** Metadata and docs must distinguish planned/unsupported HIR/MIR families from implemented deterministic support.

**Rationale:** Operators need to know whether compiler-grounded evidence is actually present. Silent partial support would undermine receipt trust.

**Alternative:** Add aspirational docs only. Rejected because the project now uses receipt contracts and gates for durable evidence claims.

## Risks / Trade-offs

**Toolchain churn** → Mitigate by requiring rustc/toolchain identity and context fields in future supplements.

**Semantic overclaiming** → Mitigate by requiring boundary fields and scenarios that explicitly reject correctness and semantic-equivalence claims.

**Implementation complexity** → Mitigate by scoping this change to the contract scaffold only; implementation can be split into later DTO/fixture/gate changes.

## Validation Plan

- Validate this change strictly with OpenSpec.
- Validate all specs strictly.
- Run `git diff --check` for whitespace and archive-readiness hygiene.
