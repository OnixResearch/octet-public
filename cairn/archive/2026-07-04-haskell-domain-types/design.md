## Context

Haskell's useful contribution here is algebraic modeling: explicit variants, newtypes for domain identities, and constructors that validate invariants once. Octet should translate that into plain Rust types with boring APIs. The goal is reviewable correctness, not abstraction density.

## Decisions

### 1. Inventory before migration

**Choice:** Start with an inventory of raw identities and state flags at core boundaries before replacing types.

**Rationale:** The highest value targets are the identities that cross module boundaries or influence diagnostics, evidence, config, and release decisions. A broad mechanical newtype sweep would create churn without proving bug reduction.

### 2. Preserve wire and user-facing compatibility

**Choice:** Typed cores SHOULD sit behind adapters that preserve existing serialized DTOs, config keys, lint names, source attributes, SARIF fields, and CLI text unless a later migration explicitly changes those contracts.

**Rationale:** Domain types should prevent internal mistakes without silently breaking downstream consumers.

### 3. Prefer small transparent Rust types

**Choice:** Use small `struct` wrappers, enums, and explicit constructors with `Result`/diagnostic returns. Avoid lens-style APIs, typeclass-inspired generic layers, macro-heavy encodings, or type-level proofs unless they replace a concrete existing failure mode.

**Rationale:** Octet needs Tiger Style clarity: explicit control flow, high assertion density, and readable review surfaces.

### 4. Validate refined values at construction

**Choice:** Non-empty collections, canonical names, absolute or repo-relative paths, sorted/deduplicated sets, and threshold/count values SHOULD be constructed through named functions that check invariants and return deterministic diagnostics.

**Rationale:** A validated value lets downstream pure functions assume the invariant without scattering defensive checks.

### 5. Keep Rust-derived DTO ownership

**Choice:** Runtime receipts and emitted evidence DTOs remain Rust-owned. Nickel contracts may be generated from those Rust sources, but domain-type work must not hand-author a second source of truth for emitted wire shapes.

**Rationale:** This follows the repo config/receipt source-of-truth policy.

## Risks / Trade-offs

- More types can make call sites noisier if adapters are not placed at clean seams.
- A partial migration can leave mixed raw/typed APIs; the inventory should name which seams are in scope for each implementation pass.
- Constructors must produce useful diagnostics, not just opaque parse failures.
