## Context

Function Object v1 remains source-token based. The archived HIR/MIR supplement contract says future HIR/MIR evidence is optional, toolchain-scoped, and supplemental. No extractor exists yet.

## Goals / Non-Goals

**Goals:**

- Surface planned HIR and MIR evidence families in Rust-derived contract metadata.
- Make unsupported status explicit and deterministic.
- Name required future fields and boundaries without claiming implementation support.

**Non-Goals:**

- Implement HIR or MIR extraction.
- Change Function Object v1 object hashes.
- Claim correctness, semantic equivalence, or cross-toolchain stability.

## Decisions

### 1. Additive contract metadata field

**Choice:** Add a `supplemental_evidence` list to `octet.object-corpus-contract-metadata.v1`.

**Rationale:** A list supports multiple evidence families while preserving backward-compatible additive JSON.

**Alternative:** Add top-level `hir` and `mir` objects. Rejected because a list better supports future THIR or verifier-specific supplements.

**Implementation:** Each family records `family`, `schema`, `schema_version`, `status`, `relationship_to_v1`, `toolchain_scope`, `required_fields`, `unsupported_reason`, and boundary booleans.

## Risks / Trade-offs

**Overclaim risk** -> Keep status `unsupported-planned`, state no extractor exists, and test boundary booleans plus human output.
