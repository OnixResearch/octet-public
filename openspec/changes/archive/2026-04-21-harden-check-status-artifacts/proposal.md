## Why

Aspen exposed two tooling-contract failures in `cargo tigerstyle check`.
First, a clean structured run (`--output-format sarif`) produced `SARIF_RESULTS=0`
and `Findings: 0` yet still exited `1`, which made downstream automation and
review treat a clean run as a failure. Second, the tool emits findings but not a
stable run-status artifact, so reviewers had to reconstruct what happened from
summary text plus detached logs instead of reading one self-describing proof
bundle.

The current code path in `cargo-tigerstyle/src/engine.rs` still collapses
`!status.success()` and `!findings.is_empty()` into the same exit code, and
`tests/consumer_integration.rs` still expects compilation-error JSON/SARIF runs
to exit `1` even though the consumer-integration contract already reserves exit
`2` for integration/configuration failures. We should harden the runner surface
before more downstream repos build automation on the wrong behavior.

## What Changes

- Restore a typed run-outcome contract in `cargo-tigerstyle` so clean and
  warning-only runs exit `0`, error-level lint findings exit `1`, and
  cargo/config/bootstrap failures exit `2`.
- Add first-class run artifact output (`--artifact-dir`) with machine-readable
  `status.json`, stable summary/output sidecars, and self-describing SARIF
  invocation metadata.
- Update contract tests and consumer-facing docs so specs, tests, docs, and
  runtime behavior agree on exit codes, artifact bundles, and the supported
  temp-workspace launch path.

## Non-Goals

- No new lint rules, autofix heuristics, or severity-policy changes.
- No redesign of OpenSpec workflow, Aspen-specific helper scripts, or
  downstream crate-test selection outside the runner/artifact contract.

## Capabilities

### Modified Capabilities

- `consumer-integration`: `cargo tigerstyle check` now publishes deterministic
  run-outcome classification across clean, warning-only, lint-failure, and
  integration-failure paths, plus optional reviewable run artifacts.

## Impact

- **Files / artifacts**: `cargo-tigerstyle/src/cli.rs`, `cargo-tigerstyle/src/engine.rs`, `cargo-tigerstyle/src/output.rs`, `tests/consumer_integration.rs`, `README.md`, `docs/consumer-guide.md`, `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/`, and `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/specs/consumer-integration/spec.md`.
- **Verification / evidence**: save consumer-integration transcripts for clean structured runs, compile-failure runs, artifact-dir bundles, SARIF invocation metadata, a docs-contract check, and a design-stage gate transcript under `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/`.
