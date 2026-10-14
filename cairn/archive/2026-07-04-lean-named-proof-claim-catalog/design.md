## Context

Octet already uses stable IDs such as `r[...]` requirement markers and proof-card property IDs. The Lean lesson is to make named claims discoverable and stable across refactors. This change creates an explicit catalog contract rather than relying on scattered strings.

## Decisions

### 1. Use a typed human-authored catalog at the boundary

**Choice:** The catalog SHOULD be human-authored in typed Nickel or an equivalent checked policy input, exported to deterministic JSON for Rust validation.

**Rationale:** Claim IDs are review policy. Nickel contracts catch malformed entries before runtime consumers rely on them.

### 2. Keep runtime validation in Rust-owned pure cores

**Choice:** Runtime code SHOULD consume checked-in generated catalog artifacts and validate references in pure Rust cores over in-memory records.

**Rationale:** Octet should not execute Nickel dynamically in the runtime path, and reference validation should be unit-testable.

### 3. Catalog claims by scope and evidence tier

**Choice:** Each claim SHOULD name its source requirement or policy, checked scope, expected evidence tier, allowed proof-card or receipt sources, owner surface, lifecycle status, and non-claims.

**Rationale:** A stable name is only useful if reviewers can see what it actually covers and what it does not cover.

### 4. Support explicit aliases, not silent renames

**Choice:** Renames SHOULD use explicit alias or deprecation entries that point at the replacement claim and preserve old references until removed by a reviewed change.

**Rationale:** Lean theorem names can churn; Octet should make that lifecycle visible and fail closed on stale references.

## Risks / Trade-offs

- A catalog can become busywork if every tiny helper is named. Start with claims that cross documentation, proof-card, proof-ref, suppression, or release-evidence boundaries.
- Alias support must not become a way to hide incompatible claim changes.
- The catalog must not imply formal proof for lint, policy, manual-review, or imported evidence tiers.
