## Context

Octet now has repo-local Cairn tooling via `github:onixresearch/cairn`, and the Valence evidence ladder includes object-corpus, THIR, MIR, fresh-build determinism, and supplement compatibility rails. The current head evidence page was refreshed manually after the supplement compatibility gate landed. That proved the page format is useful, but the process remains fragile: a future commit can change the aggregate rail set or payload commit without updating the page.

## Decisions

### 1. Treat the Markdown page as generated review surface

**Choice:** Implement a deterministic generator/checker for `docs/current-head-valence-evidence.md` rather than relying on hand edits.

**Rationale:** The page is a human-readable review artifact, not a source of truth. The source of truth should be repository state plus declared evidence rail inventory and observed pass signals. A generator makes stale payload hashes and missing rails mechanically visible.

### 2. Keep payload commit separate from evidence-recording commit

**Choice:** The generated page MUST record the verified payload commit independently from the later commit that records the page.

**Rationale:** Current-head evidence pages should avoid self-referential claims. The payload commit is the code/evidence state that was audited; the docs commit is only the record of that audit.

### 3. Fail closed on drift and incomplete evidence

**Choice:** The check path MUST fail when the committed page omits an expected rail, changes non-claim boundaries, records the wrong payload commit, or cannot prove the aggregate pass signal inventory.

**Rationale:** Silent drift defeats the value of the evidence index. A mismatch should block future closeout until the generator output and checked-in page are reconciled.

### 4. Keep claims bounded

**Choice:** The generator MUST preserve explicit non-claims for correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, and general HIR/THIR/MIR extraction support.

**Rationale:** The current evidence ladder is selected-corpus / selected-fixture replay evidence. Automation should reduce stale docs risk without broadening product claims.

## Risks / Trade-offs

- The first generator may encode the current rail inventory explicitly. That is acceptable if the staleness check fails when the inventory changes.
- Running every underlying rail inside the generator could make the check too expensive. Prefer composing an already-run aggregate rail or checking deterministic captured output when practical.
- The page should remain readable; do not replace the Markdown index with opaque JSON-only evidence.
