## Design

The source-boundary core is a pure Rust module over in-memory source descriptors and digest inventories. It does not call Git, walk files, fetch registries, inspect Cargo metadata, or publish artifacts. Octet shells and existing source-acquisition rails collect facts and then pass them to the core.

### Decisions

1. **Source kind is typed.** Git, local worktree, vendored snapshot, registry package, generated artifact, and unknown/manual-review source kinds carry distinct required fields.

2. **Dirty-state policy is explicit.** A local or Git source with uncommitted changes may pass only when the descriptor carries an allowed dirty-state policy and bounded non-claim.

3. **Vendoring is receipt-backed.** Vendored snapshots require upstream source identity, revision, selected paths, digest inventory, and local-patch disposition.

4. **Registry and generated sources stay bounded.** Registry package identity and generated artifact identity are supported as source descriptors, but the core does not fetch or regenerate them.

5. **No trust promotion.** Passing source-boundary validation proves descriptor shape, source identity completeness, digest inventory consistency, and non-claim conformance only.

### Validation shape

Positive tests cover committed Git sources, approved local overrides, aligned vendored snapshots, registry package descriptors, and generated artifact descriptors. Negative tests cover missing revision, dirty disallowed source, path escape, missing digest inventory, stale vendored digest, unknown source kind, missing non-claims, and text claiming source-boundary validation proves dependency trust or code correctness.
