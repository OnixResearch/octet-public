## Context

Octet now emits content-addressed Rust evidence as deterministic JSON across function objects, object bundles, corpus receipts, corpus receipt checks, schema-hardened invalid-input reports, and the checked-in internal corpus gate fixture. OpenSpec changes and evidence notes are Markdown. These formats are useful because they are reviewable, diffable, and easy to parse in CI.

The missing contract is ownership: JSON is the current interchange encoding, not the semantic source of truth by itself. Runtime receipt shape should follow Rust DTOs and explicit schema/version fields. Human-authored policy or catalog decisions should use Nickel-first contracts where this repo already expects them.

## Goals

- Keep Octet's human review artifacts in Markdown.
- Keep Octet's current machine receipts and fixtures in deterministic JSON.
- Make Rust DTOs and explicit schema versions the runtime compatibility source of truth for emitted receipts.
- Use Nickel-authored policy/catalog metadata for human-composed compatibility registration where applicable.
- Preserve evidence-only boundary language.

## Non-Goals

- Replacing existing JSON CLI output.
- Adding a binary encoding now.
- Moving Octet receipt ownership to Cairn.
- Claiming correctness, semantic equivalence, or cross-toolchain determinism from matching hashes.

## Decisions

### 1. Markdown remains the human review surface

**Choice:** Keep OpenSpec proposals, designs, tasks, archived evidence notes, and reviewer context in Markdown.

**Rationale:** These artifacts carry rationale, caveats, and review context. Markdown is readable in diffs and archives and should not be optimized for machine replay.

### 2. Deterministic JSON remains the v1 receipt surface

**Choice:** Keep function objects, object bundles, corpus receipts, corpus check reports, invalid-input reports, and checked-in fixtures as deterministic JSON for v1.

**Rationale:** JSON is easy to inspect, diff, parse from CI, include in bundles, and compare as stable fixture bytes when Octet controls ordering and hash material.

### 3. Rust DTOs own emitted receipt shape

**Choice:** Treat Rust structs/serde behavior plus explicit schema/version identifiers as the runtime source of truth for emitted receipt compatibility.

**Rationale:** Octet emits the receipts from Rust. Hand-maintained JSON schema as the normative source can drift from the implementation; schema metadata should be generated or registered from the implementation boundary where practical.

### 4. Nickel tracks human-authored policy/catalog registration

**Choice:** When humans compose compatibility policy, receipt-family catalogs, or config around receipt families, use Nickel-authored metadata and register the family as Rust-derived, Nickel-authored, or non-candidate.

**Rationale:** This matches the repo policy: Nickel for human-authored config/contracts, Rust-derived contracts for emitted receipts, and explicit exclusions for non-candidates.

### 5. Future encodings are additive

**Choice:** Future binary/packed formats may be added for transport, signing, size, or speed, but they must be additive to v1 JSON unless a later migration explicitly changes the review surface.

**Rationale:** JSON fixtures are currently valuable for review while the object model evolves. A binary encoding should not erase the reviewable surface by accident.

## Risks / Trade-offs

- **JSON fixture churn**: DTO evolution can change fixture bytes. Mitigation: schema/version fields plus explicit fixture refresh in the same review.
- **Overclaiming hashes**: Stable bytes can be mistaken for semantic proof. Mitigation: keep no-correctness/no-semantic-equivalence fields and spec language mandatory.
- **Contract duplication**: Nickel and Rust can drift. Mitigation: Rust-derived receipt contracts should not be hand-authored as normative JSON schema when generated/registered metadata can represent them.
