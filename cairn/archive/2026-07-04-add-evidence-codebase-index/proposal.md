## Why

Unison's most useful lesson for Octet is not a new language runtime; it is the codebase-as-index model. Octet already emits fingerprints, receipts, transcripts, provenance records, proof cards, policy bundles, and release smoke artifacts, but reviewers still have to stitch those surfaces together by reading files and command output manually. A first-class evidence index would give Octet a local, content-addressed map from requirement, lint, policy, finding, source object, and artifact identities to the concrete files and replay commands that support them.

## What Changes

- Add an Octet-owned evidence codebase index that ingests existing Octet evidence artifacts without changing their source formats.
- Add deterministic query/search surfaces for requirement IDs, lint IDs, policy IDs, fingerprints, artifact hashes, replay capsules, and provenance records.
- Add a static HTML review surface generated from the same index so reviewers can navigate finding → lint reference → source fingerprint → config → transcript/proof/receipt without trusting a hosted service.
- Keep the index as review/provenance infrastructure only: it must not claim correctness, semantic equivalence, complete dependency closure, or Unison-style executable code mobility.

## Impact

- **Files / artifacts**: expected implementation areas are `cargo-octet/src/`, `crates/valence-core/` if a pure index model is shared, docs under `docs/`, checked examples under `docs/examples/`, and Nix/documentation checks.
- **Ordering / dependencies**: build the pure index schema and normalization before CLI query/rendering; add docs/examples after CLI output contracts stabilize.
- **Testing**: positive fixture with multiple artifact families, negative fixtures for malformed/missing/stale artifact identity, deterministic JSON snapshot tests, HTML smoke checks, Cairn validate/gates, and docs-example replay.
- **Compatibility**: existing artifact formats remain valid; the index is additive and consumes current surfaces.
