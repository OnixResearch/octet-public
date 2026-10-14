## Context

The current `valence-core` README lists the original evidence-index and Function Object surfaces, while `src/lib.rs` also exposes `evidence_codebase_index`. The existing readiness rail checks package metadata and public modules, but it does not prove the README/API inventory stays synchronized with exports or that the package can still pass focused tests and documentation generation as the seam grows.

## Decisions

### 1. Treat documentation inventory as part of the seam contract

**Choice:** The readiness rail MUST assert that the `valence-core` README, crate-level docs, and exported public modules name the same supported in-repository surfaces.

**Rationale:** A future extraction depends on clear ownership. Stale docs make it unclear whether a module is Valence-owned, Octet-owned-but-hosted, or accidental.

### 2. Keep the rail package-readiness scoped

**Choice:** The rail SHOULD compose focused `valence-core` tests, documentation generation, package listing, metadata checks, module inventory checks, and boundary/non-claim checks without publishing or moving release ownership.

**Rationale:** This gives stronger local evidence for the seam without turning it into a standalone release gate.

### 3. Fail closed on missing or extra inventory entries

**Choice:** The rail MUST fail if a public module is missing from README/crate docs, if README names a removed public module, or if required no-cutover/non-claim language disappears.

**Rationale:** The problem is drift. Both missing and stale entries are review hazards.

### 4. Keep Octet as the shell

**Choice:** Documentation must keep source scanning, filesystem reads, CLI parsing, stdout/stderr, marker mutation, corpus orchestration, supplement orchestration, process exits, repository tags, and package publication outside `valence-core`.

**Rationale:** This change improves the seam contract; it does not broaden the seam.

## Risks / Trade-offs

- The rail becomes slightly more expensive because it runs focused tests/docs/package listing, but it is still much narrower than the aggregate Valence evidence gate.
- Module inventory assertions can become noisy if public modules churn; the failure is intentional because public API drift should be reviewed.
