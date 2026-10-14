# Design: Valence core package readiness rail

## Boundary

`valence-core` remains an in-repository crate owned by the Octet workspace. This slice makes the crate more package-shaped, but it does not make it a standalone project or release artifact. Octet continues to own CLI argument parsing, filesystem access, source scanning, object marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, process exits, evidence bundle release ownership, and repository tags.

The readiness rail is intentionally metadata/API-boundary evidence:

- `crates/valence-core/Cargo.toml` carries package metadata suitable for downstream inspection.
- `crates/valence-core/README.md` states the current crate surfaces and no-cutover/non-claim boundaries.
- `crates/valence-core/src/lib.rs` documents the same boundary and exposes the current public modules.
- `flake.nix` exposes `packages.${system}.valence-core` and `checks.${system}.valence-core-package-readiness`.

## Verification

Verification must demonstrate:

- `valence-core` unit tests still pass.
- `cargo doc -p valence-core --no-deps` succeeds for API documentation generation.
- `cargo package -p valence-core --allow-dirty --no-verify --list` succeeds and includes the README/source files without package metadata warnings.
- The named Nix check `valence-core-package-readiness` builds.
- The aggregate `valence-object-evidence` gate still includes and passes the package-readiness rail.
- Cairn validation and proposal/design/tasks gates pass.

## Non-claims

This rail proves only in-repository package readiness for the current `valence-core` crate boundary. It is not package publication, standalone repository readiness, runtime compatibility with an external consumer, behavioral correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, cross-machine determinism proof, cross-toolchain determinism proof, or general HIR/THIR/MIR extraction proof.
