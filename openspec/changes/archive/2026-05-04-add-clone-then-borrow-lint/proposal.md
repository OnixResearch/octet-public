## Why

Rust Design Patterns calls out cloning to satisfy the borrow checker as a smell: the clone often hides unclear ownership structure and adds avoidable allocation or copy cost. Tigerstyle already imports that guidance in `docs/rust-idiom-guidance.md`, but it lacks a focused lint plan for the lowest-noise subset.

The next high-ROI child lint is the narrow case where code clones a value and then immediately borrows the clone. A borrowed clone usually communicates neither ownership transfer nor independent mutation; it pays clone cost only to feed an API that still receives a borrow.

## What Changes

- **Future lint:** Add `clone_then_borrow` as an advisory performance lint.
- **Initial signal:** Detect immediate borrowed clone temporaries such as `use_name(&name.clone())` and `checksum(&bytes.clone())`.
- **Guidance:** Prefer borrowing the original value, narrowing the original borrow, or using explicit adapters such as `.as_ref()`, `.as_str()`, `.as_slice()`, or `.as_path()` where those communicate intent.
- **Boundaries:** Do not warn on clones that are moved into ownership-consuming APIs, stored for later independent lifetime, mutated separately, or required to cross ownership/thread/lifetime boundaries.

## Capabilities

### New Capabilities

- `performance.clone-then-borrow-lint`: future lint coverage for avoidable clone-then-borrow expressions.

### Modified Capabilities

- `performance`: extends performance guidance from review-only clone avoidance into a fixture-backed advisory lint candidate.

## Impact

- **Files:** Future implementation will touch `src/performance/`, `src/lib.rs`, diagnostics metadata, UI fixtures, lint catalog docs, and rollout config examples.
- **APIs:** No public runtime API change; new Dylint lint name will be `clone_then_borrow` if implemented.
- **Dependencies:** No new dependencies expected.
- **Testing:** Validate with compliant, noncompliant, and false-positive-boundary UI fixtures plus package checks before archive.
