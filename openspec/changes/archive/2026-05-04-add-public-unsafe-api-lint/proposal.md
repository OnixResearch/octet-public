## Why

Rust Design Patterns recommends contained unsafe modules: unsafe code should be small, audited, and hidden behind safe APIs where possible. Tiger Style already says public APIs should stay safe and narrow, but the lint suite does not yet surface public `unsafe fn` boundaries that push proof obligations onto callers.

The next high-ROI child lint is the narrow case of public Rust `unsafe fn` items and inherent methods. Those signatures are review-critical because every downstream caller must uphold undocumented or hard-to-test invariants.

## What Changes

- **Future lint:** Add `public_unsafe_api` as an advisory safety lint.
- **Initial signal:** Detect public free functions and public inherent methods declared `unsafe fn` under the normal Rust ABI.
- **Guidance:** Prefer a safe public wrapper around a small private unsafe helper, with safety invariants localized near the unsafe block.
- **Boundaries:** Do not warn on private unsafe helpers, trait-dictated methods, FFI/export shims, or test-only fixtures in the first implementation.

## Capabilities

### New Capabilities

- `safety.public-unsafe-api-lint`: future lint coverage for public unsafe Rust APIs that leak unsafe obligations to callers.

### Modified Capabilities

- `safety`: extends unsafe-containment guidance from review-only process into a fixture-backed advisory lint candidate.

## Impact

- **Files:** Future implementation will touch `src/safety/`, `src/lib.rs`, diagnostics metadata, UI fixtures, lint catalog docs, and rollout config examples.
- **APIs:** No public runtime API change; new Dylint lint name will be `public_unsafe_api` if implemented.
- **Dependencies:** No new dependencies expected.
- **Testing:** Validate with compliant, noncompliant, and false-positive-boundary UI fixtures plus package checks before archive.
