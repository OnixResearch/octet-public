## Context

Octet has separate evidence surfaces for lints, standards policies, fingerprints, replay capsules, transcripts, proof cards, provenance, and release smoke checks. The current trust-boundary docs describe those surfaces, but there is no single deterministic index that answers: "what evidence supports this requirement or finding, and how do I replay it?" Unison Share shows the value of a browsable, content-addressed codebase graph; Octet can adapt that as a local evidence graph over Rust artifacts and receipts.

## Decisions

### 1. Index existing evidence, do not replace existing artifacts

**Choice:** The index MUST consume existing JSON, SARIF, JSONL, manifest, proof-card, replay-capsule, transcript, and provenance surfaces rather than becoming the canonical storage format for each artifact family.

**Rationale:** Current surfaces are already used by scripts and release evidence. The index should improve discoverability and review without breaking consumers.

### 2. Put normalization in a pure core

**Choice:** Artifact normalization, key extraction, graph construction, sorting, issue classification, and query evaluation SHOULD live in deterministic pure functions over in-memory DTOs. CLI code owns filesystem reads, output rendering, and exit codes.

**Rationale:** The index is primarily a deterministic transform. Keeping it pure makes positive and negative fixtures cheap to test and prevents hidden dependence on clocks, local paths, or environment state.

### 3. Use BLAKE3 identities and explicit schema versions

**Choice:** Index records MUST include schema version, artifact family, normalized stable IDs, BLAKE3 content identity for deterministic artifacts, source locator, and optional parent/child links.

**Rationale:** Octet already standardizes on BLAKE3 for evidence identity. The index should remain comparable across machines while excluding unstable local details.

### 4. Ship two review surfaces over one model

**Choice:** Provide a machine-readable query/report surface and a generated static HTML surface from the same index model.

**Rationale:** Scripts need JSON; humans need browsable review trails. One model avoids two diverging interpretations of the same evidence graph.

### 5. Preserve non-claims

**Choice:** Every generated summary MUST state that the index proves artifact identity and linkage only. It MUST NOT claim whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, or formal proof unless a linked formal-proof receipt makes that scoped claim.

**Rationale:** A better map of evidence is not stronger evidence by itself.

## Risks / Trade-offs

- Some historical artifacts may lack enough metadata for full graph links. The index should record partial coverage with explicit caveats instead of guessing.
- HTML output can become noisy. Start with focused pages for requirement, lint, artifact, and finding views.
- Broad repository scans can become expensive. Prefer explicit artifact directories and manifests before recursive discovery.
