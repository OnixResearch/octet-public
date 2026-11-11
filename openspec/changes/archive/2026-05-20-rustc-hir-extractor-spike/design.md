## Context

Octet has Function Object v1 evidence and a HIR/MIR supplement pilot. The pilot is intentionally source-token bounded. It records schema/status/toolchain/extractor fields and fail-closed comparisons, but it does not ask rustc for HIR.

## Goals / Non-Goals

**Goals:**
- Extract one deterministic HIR observation from a selected fixture function using the pinned rustc/toolchain available in the Nix devshell.
- Attach the HIR observation to the existing Function Object v1 hash.
- Persist exact toolchain and extractor identity in the supplement receipt.
- Fail closed on any replay context mismatch.

**Non-Goals:**
- Full HIR support for arbitrary crates.
- THIR or MIR extraction.
- Type-theoretic equivalence, behavioral correctness, pure-cache eligibility, or cross-toolchain stability.
- Replacing source-token Function Object v1 identity.

## Decisions

### 1. Fixture-first rustc HIR spike

**Choice:** Implement the first rustc-backed HIR path for a tiny checked-in fixture only.

**Rationale:** The fixture boundary keeps implementation and evidence reviewable while proving the rustc integration seam.

**Alternative:** Build a general workspace HIR extractor first. Rejected because it would enlarge scope, destabilize replay evidence, and invite overclaims.

**Implementation:** Add a narrow extractor helper and tests that use a pinned fixture source path and function name. Receipts must include fixture path/function context and exact rustc/toolchain identity.

### 2. Preserve Function Object v1 identity

**Choice:** HIR output remains an attached supplement to `octet.function-object.v1`.

**Rationale:** Existing object hashes already define review/provenance identity. HIR evidence is additional context, not a new canonical identity.

**Alternative:** Recompute object identity from HIR. Rejected because HIR is compiler-version-coupled and not cross-toolchain stable.

### 3. Fail-closed replay contract

**Choice:** The spike must add negative checks for mismatched toolchain/context/extractor/schema/observation fields before any support status can strengthen.

**Rationale:** Negative evidence prevents accidental support overclaim and protects downstream gates.

**Alternative:** Only test the positive fixture. Rejected because a positive receipt alone does not prove replay boundaries.

## Risks / Trade-offs

**rustc-private churn** → keep the spike fixture-scoped, pin toolchain identity, and record unsupported caveats.

**Overclaiming support** → keep names/docs as `spike` or `fixture-backed`; do not generalize contract status beyond deterministic fixture evidence.

**Fixture drift** → save evidence artifacts with exact commands and add tests that compare stable receipt fields rather than ambient build paths.
