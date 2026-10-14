## Why

Octet should not compete with Clippy for general Rust hygiene. Clippy already covers a broad base of correctness, style, complexity, and performance lints; Octet should focus on Tiger Style systems constraints while preserving Clippy findings as first-class evidence when a consumer opts in.

A Clippy co-reporting rail makes the relationship clear: Clippy findings remain Clippy findings, Octet findings remain Octet findings, and both can appear in one artifact directory with deterministic provenance.

## What Changes

- Add a co-reporting rail that runs or imports Clippy JSON output under an explicit option.
- Preserve Clippy lint names, levels, spans, and messages without rewriting them as Octet diagnostics.
- Emit `static_lint` provenance with `checker_id = "clippy"` or the registered external-lint checker identity.
- Document Clippy as a complementary hygiene layer rather than an Octet replacement.

## Non-Goals

- Do not fork or vendor Clippy lints.
- Do not enable every Clippy group for consumers by default.
- Do not collapse Clippy findings into Octet lint IDs.
- Do not make hosted code-scanning platforms mandatory.

## Impact

- **Files**: runner output code, artifact schemas, docs, tests, policy registry checker IDs, and Cairn specs.
- **Behavior**: optional Clippy findings appear in Octet artifact bundles and can be verified with provenance.
- **Testing**: JSON import fixture, run-mode fixture, malformed Clippy JSON failure, duplicate-source handling, artifact verification, and Cairn gates.
