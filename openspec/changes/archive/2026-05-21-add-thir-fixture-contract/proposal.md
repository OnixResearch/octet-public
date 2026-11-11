## Why

Octet now exposes `--family thir` as a fail-closed runtime seam, but every THIR receipt remains `unsupported-planned`. The next product step is a tiny, deterministic fixture-backed contract so operators can see exactly what a bounded THIR supplement support claim looks like without implying general THIR extraction, semantic correctness, or proof support.

## What Changes

- Add a closed THIR fixture scope for one checked-in Rust function shape.
- Permit `pilot-supported` only when `--family thir` uses that exact fixture scope/shape.
- Keep default/non-fixture THIR requests `unsupported-planned` and fail closed.
- Update contract metadata, CLI help, docs, tests, and evidence to name the fixture boundary.

## Scope

In scope:
- One deterministic THIR fixture selector: `checked-arithmetic`.
- One exact fixture/function pair: `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`.
- Emit/check roundtrip and negative mismatch evidence for that fixture-backed case.
- Contract metadata and docs that distinguish fixture-backed support from general THIR support.

Out of scope:
- General THIR extraction for arbitrary Rust code.
- Semantic equivalence, correctness, pure-cache eligibility, or cross-toolchain stability claims.
- Broad Valence chemistry/fractal/valid-bonds semantics.

## Impact

- Files: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/`, `cargo-octet/tests/fixtures/thir-spike/checked.rs`, `docs/`, and `openspec/specs/standards-codification/spec.md`.
- API/CLI: adds a bounded `--spike rustc-thir-fixture` supplement scope for THIR emit/check and fixtures inventory.
- Testing: focused supplement tests, contract metadata tests, OpenSpec gates, all-spec validation, and whitespace checks.

## Oracle Checkpoint

- Owner: repo-local Octet maintainer/agent.
- Decision: keep this delta under `openspec/specs/standards-codification/spec.md` for this slice because the existing canonical `HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]` requirement and prior archived THIR fail-closed change already own the supplement metadata/CLI contract in that spec family.
- Next action: do not create a second spec home for the same supplement contract in this change; if a future cleanup splits Octet supplement contracts out of standards-codification, migrate the full requirement and all scenarios together.
