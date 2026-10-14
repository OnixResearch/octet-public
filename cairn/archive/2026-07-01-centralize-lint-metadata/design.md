## Context

The lint crate already has strong inventory and diagnostic snapshot tests, but the runner and docs carry separate hand-authored metadata. That duplication caused observable drift: registered lints are absent from docs and from `cargo-octet` sidecar metadata, while the documented canonical namespace does not match every registered tool group and fixture.

## Decisions

### 1. Rust owns the lint catalog

**Choice:** Create one Rust-owned catalog source for lint identity, group, default level, principle, guidance, effect family, docs summary, and namespace aliases.

**Rationale:** The lint inventory is Rust code and rustc lint registration is the behavior that matters. Keeping the catalog in Rust avoids runtime Nickel dependencies and lets unit tests compare registration, runner output, and generated docs in memory.

### 2. Runner metadata is derived or fixture-checked

**Choice:** Make `cargo-octet` consume generated metadata or a shared normal Rust crate instead of maintaining an independent `lint_meta_map()` table.

**Rationale:** Runner sidecars and SARIF are release evidence. Missing metadata should be impossible when a lint is registered.

### 3. Documentation is freshness-checked

**Choice:** Either generate the lint catalog sections or add deterministic tests that compare docs against the Rust-owned catalog.

**Rationale:** Fully generated docs reduce drift, but freshness tests are an acceptable first step when prose around generated tables must remain hand-authored.

## Risks / Trade-offs

- A new shared crate may need care to avoid pulling `rustc_private` into `cargo-octet`. Mitigate by making the catalog crate dependency-free and data-only.
- Generated docs can become noisy. Mitigate by limiting generation to catalog tables and keeping surrounding explanatory prose hand-authored.
- Namespace migration may overlap with the namespace change package. Mitigate by making this package enforce the final selected namespace rather than choosing it.

## Validation

- Positive: every registered lint has metadata, appears in runner output metadata, and appears in docs with the correct group and default level.
- Negative: adding a fake registered lint without metadata, removing one docs entry, or omitting runner metadata fails the targeted freshness test.
- Regression: `cargo octet check --output-format json` and SARIF fixture tests preserve metadata fields for all emitted lints.
