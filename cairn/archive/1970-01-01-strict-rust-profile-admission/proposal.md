## Why

Octet already has the lint catalog, purity checks, function-address receipts, profile metadata, and evidence bundle needed for a stricter Rust-compatible path, but consumers must currently assemble those controls by hand in `dylint.toml` and command-line flags. That makes the intended assurance posture implicit, allows profile-critical lints to be accidentally disabled, and leaves `profile_name` fixed to `workspace-metadata` even when an operator expects a named checked path.

A bounded strict-profile admission surface gives consumers one explicit selection that activates a reviewed minimum lint set, validates required core scopes, raises function-address checking to the profile floor, and binds the selected profile into deterministic run evidence. This is the first enforceable slice of stricter Rust; it deliberately remains ordinary Rust plus source-shape policy rather than claiming a new language, transitive effect inference, or whole-program proof.

## What Changes

- Add a Nickel-authored strict-profile catalog with checked-in generated runtime JSON for the `robust`, `deterministic-core`, and `bounded-systems` profiles.
- Add `cargo octet check --profile <PROFILE>` selection with deterministic rejection of missing or unknown profile IDs.
- Apply each selected profile's required lint set at `deny` even when consumer `dylint.toml` disables or weakens those lints; explicit source-level suppressions remain governed by Octet's existing justification lints.
- Require configured `core_source_scopes` for `deterministic-core` and fail before Cargo execution when the profile cannot identify its intended core.
- Raise function-address mode and default scope to each profile's declared floor without weakening an explicitly stronger operator selection.
- Bind the selected profile ID and BLAKE3 policy identity into status, summary, provenance checked scope, and existing artifact verification.
- Document bootstrap, compatibility, exact enforced claims, and explicit non-claims.

## Impact

- **Files / artifacts**: `standards/strict-profiles/`, `standards/policies/registry.ncl`, `crates/octet-lint-metadata`, the lint-driver config loader, `cargo-octet` CLI/engine/determinism/function-address integration, focused fixtures/tests, `README.md`, and a strict-profile operator guide.
- **Ordering / dependencies**: define and validate the Nickel catalog first; load it through the shared metadata crate; then wire CLI selection, driver enforcement, function-address floors, and run evidence.
- **Testing**: positive tests cover every shipped profile, deterministic selection, lint-floor strengthening, core-scope resolution, function-address floors, and metadata binding. Negative tests cover malformed Nickel, unknown profiles, missing core scopes, stale generated JSON, and attempts to weaken required lints through workspace config.
- **Compatibility**: `cargo octet check` without `--profile` retains existing workspace-metadata behavior. Existing source-level justified suppression semantics remain unchanged. The change does not add Rust syntax, MIR call-graph closure, transitive effect proof, formal-proof orchestration, dependency-closure certification, or release eligibility.
