## Context

The evidence-codebase index capability ingests existing Octet evidence artifacts and emits deterministic JSON/query/HTML review surfaces. The pure implementation currently lives in `valence-core::evidence_codebase_index`, while the CLI shell lives in `cargo-octet`. The generated title and boundary are Octet-specific. This is acceptable as an in-repository seam only if the ownership contract is explicit and the API does not imply standalone Valence ownership prematurely.

## Decisions

### 1. Name product identity explicitly

**Choice:** Evidence-codebase index docs, generated JSON, generated HTML, and readiness checks MUST identify the current capability as Octet-owned evidence indexing unless a future reviewed change parameterizes product identity or moves the code to a different crate.

**Rationale:** The current evidence artifacts, CLI command, and specs are Octet-owned. Hosting pure functions in `valence-core` should not silently transfer product ownership or release responsibility.

### 2. Record hosted-core status in Valence docs

**Choice:** `valence-core` docs SHOULD describe `evidence_codebase_index` as a pure deterministic core currently used by Octet evidence surfaces, with Octet retaining CLI, filesystem traversal, artifact discovery, output writing, and release/gate ownership.

**Rationale:** This gives reviewers a clear map without forcing an immediate crate move.

### 3. Decompose implementation by role

**Choice:** The current large module SHOULD be split behind the same public API into model, ingest, query, HTML, and boundary helpers. All submodules should remain pure and deterministic.

**Rationale:** Smaller modules make hash/identity, artifact-family parsing, graph linking, and rendering decisions independently reviewable.

### 4. Preserve output and public API compatibility

**Choice:** Existing CLI behavior, JSON schema, query output, HTML non-claim boundary, deterministic sorting, and issue severity semantics MUST remain compatible unless a separate reviewed migration changes the schema.

**Rationale:** The goal is seam clarity and maintainability, not a behavior break.

### 5. Strengthen negative tests around ownership and overclaims

**Choice:** Tests SHOULD prove that generated HTML/JSON do not claim Valence standalone ownership, whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, or formal proof for non-proof evidence tiers.

**Rationale:** The ownership ambiguity is a claim-boundary problem, so negative tests need to target overclaims directly.

## Risks / Trade-offs

- Splitting a large module can create noisy diffs without behavior changes; keep re-exports stable and verify byte-stable fixtures.
- If later work decides the index belongs in a new shared crate rather than `valence-core`, this package should make that migration easier by isolating pure submodules first.
