## Why

external evidence tooling's proof pages distinguish SMT proof, property validation, type rejection, and human modeling decisions instead of flattening every result into "pass." Octet already emits several kinds of evidence, but findings and sidecars do not have one shared, machine-readable taxonomy that says what kind of evidence supports each result.

Octet should adopt explicit evidence tiers so governance reviewers can tell whether a result came from a static lint, a policy check, a fixture, an autofix replay, a transcript assertion, a formal proof, or a manual-review boundary.

## What Changes

- Define a Rust-owned evidence-tier enum for emitted artifacts and sidecars, with generated or checked Nickel contract coverage where applicable.
- Start with tiers: `static_lint`, `policy_check`, `ui_fixture`, `autofix_verified`, `transcript_assertion`, `formal_proof`, and `manual_review_required`.
- Require artifacts to label validation as validation and proof as proof; no tier may be silently upgraded.
- Add positive and negative schema/serialization tests for known tiers and unknown-tier rejection.

## Impact

- **Files**: runner sidecar DTOs under `cargo-octet/src/`, standards evidence DTOs under `crates/octet-standards/src/`, policy/receipt registry entries under `standards/policies/registry.ncl`, generated policy artifacts, and docs.
- **Testing**: Rust DTO tests, Nickel contract freshness checks, positive artifact fixture for each tier, negative fixture rejecting an unknown tier and a proof tier with no proof artifact identity.
