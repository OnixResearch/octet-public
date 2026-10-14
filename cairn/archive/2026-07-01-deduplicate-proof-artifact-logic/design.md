## Context

The audit found similar helper families in `crates/octet-standards/src/main.rs` and `crates/octet-standards/src/proof.rs`, with related receipt and provenance handling in `cargo-octet/src/prove.rs`. These paths are receipt-critical: a drift between producer and validator would weaken evidence guarantees.

## Decisions

### 1. Pure proof/artifact core owns canonical decisions

**Choice:** Move BLAKE3 text/file identity, canonical closure graph normalization, proof receipt binding, proof rail relevance, and generated artifact manifest comparisons into pure functions.

**Rationale:** These decisions should be deterministic and testable without CLI state, filesystem traversal, or process execution.

### 2. CLIs keep local I/O and presentation

**Choice:** Keep file loading, command execution, stdout/stderr, and JSON presentation in `cargo-octet` and `octet-standards` shells.

**Rationale:** The same core decision can serve both command surfaces while preserving their distinct user-facing behavior.

### 3. Shared tests protect equivalence

**Choice:** Add shared positive and negative fixtures for valid receipts, stale BLAKE3 bindings, malformed closure graphs, missing provenance, and unsupported schemas.

**Rationale:** The refactor should prove equivalence and fail-closed behavior, not just reduce code size.

## Risks / Trade-offs

- Moving logic across crate boundaries may require dependency changes. Mitigate by placing shared code in the lowest crate that both command surfaces can depend on without introducing CLI dependencies.
- Diagnostics may change wording accidentally. Mitigate with existing snapshot/golden tests or compatibility assertions around diagnostic codes and key substrings.
- Some helper duplication may be intentionally contextual. Mitigate by extracting only deterministic core logic and leaving context-specific presentation local.

## Validation

- Positive: valid proof receipts and artifact manifests still pass through both runner and standards paths.
- Negative: stale hashes, missing proof bindings, unsupported schemas, malformed closure graphs, and missing provenance still fail closed with deterministic diagnostics.
- Regression: existing proof rails, artifact manifest tests, and `cargo octet prove` tests pass without output contract drift.
