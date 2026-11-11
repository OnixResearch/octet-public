## 1. Canonical engine and command surface

- [x] 1.1 Add a new `cargo-tigerstyle` workspace member with internal engine modules and the `cargo-tigerstyle` binary as the supported downstream entry point
- [x] 1.2 Choose and implement the packaged tigerstyle source / artifact layout with a BLAKE3-keyed cache path and atomic extraction so `cargo tigerstyle check` does not rely on `TIGERSTYLE_PATH` as the normal consumer contract
- [x] 1.3 Implement CLI-to-lint-engine orchestration, explicit scope override handling, and explicit `cargo_check_args` replacement semantics without copied consumer scripts
- [x] 1.4 Implement internal toolchain resolution, including nix-managed nightly handling without consumer wrapper scripts
- [x] 1.5 Implement isolated `dylint_driver` bootstrap and lockfile-safe execution with snapshot / restore / fail-closed behavior
- [x] 1.6 Implement the exit-code contract: `0` for success or warning-only runs, `1` for error-level lint findings, and `2` for configuration or integration failures
- [x] 1.7 Add regression tests proving successful runs leave consumer `Cargo.lock` unchanged and fail closed after restoring it on unexpected mutation

## 2. Workspace metadata contract

- [x] 2.1 Define the `[workspace.metadata.tigerstyle]` schema for default scope and default cargo-check args
- [x] 2.2 Parse workspace metadata in the engine and make no-argument `cargo tigerstyle check` use that metadata by default
- [x] 2.3 Add tests proving valid metadata comes from `Cargo.toml`, invalid or incomplete metadata fails with a user-facing error, explicit CLI `-- ...` args replace metadata defaults, and lint tuning remains in consumer-owned `dylint.toml`

## 3. Nix integration

- [x] 3.1 Replace shell-heavy downstream Nix entry points with flake apps / check helpers that call the shared `cargo-tigerstyle` binary
- [x] 3.2 Ensure the embedded source bundle and driver bootstrap path work inside Nix vendored / sandboxed builds, not only in `nix develop`
- [x] 3.3 Add verification under `nix flake check` that `nix run` / flake checks use the same scope and policy contract as `cargo tigerstyle check`

## 4. Thin helper and documentation cleanup

- [x] 4.1 Update downstream rollout docs and examples to use the first-class CLI + workspace metadata path instead of the copied shell-wrapper workflow
- [x] 4.2 Document the thin-wrapper rule for `just`, `cargo xtask`, and CI entry points, and add at least one verification fixture under `tests/fixtures/consumer-integration/` showing a helper delegates without reimplementing policy
- [x] 4.3 Remove the historical shell-wrapper file and its superseded test surface only after equivalent CLI / Nix coverage is in place and saved evidence exists

## 5. Validation evidence

- [x] 5.1 Save disposable-consumer proof artifacts covering CLI lockfile invariants, metadata-driven default scope, explicit CLI override behavior, nix-managed nightly handling, and Nix parity before marking the change complete
