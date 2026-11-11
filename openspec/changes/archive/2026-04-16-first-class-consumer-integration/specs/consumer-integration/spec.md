## ADDED Requirements

### Requirement: First-Class Consumer Check Command

Tigerstyle MUST provide a first-party downstream integration command exposed as
`cargo tigerstyle check`.

The supported onboarding path for downstream Rust consumers MUST be that
command, not a copied tigerstyle shell script.

The command MUST:

- ship as a first-party `cargo-tigerstyle` binary so `cargo tigerstyle check`
  works without a copied consumer script
- resolve the matching tigerstyle lint source or packaged artifact internally,
  without requiring consumer-local `TIGERSTYLE_PATH` or sibling-checkout
  discovery as the normal contract
- run the tigerstyle lint suite for the configured consumer scope
- own toolchain / driver bootstrap internally instead of requiring consumer
  wrapper scripts
- keep warning-level findings as warnings while returning exit code `1` for
  error-level lint findings and exit code `2` for integration or configuration
  failures
- keep the consumer workspace `Cargo.lock` byte-for-byte unchanged after a
  successful run
- fail after restoring the original `Cargo.lock` if the check path mutates the
  lockfile unexpectedly
- preserve explicit command-line scope overrides when the caller passes them

#### Scenario: Installed cargo subcommand does not need TIGERSTYLE_PATH

- **WHEN** a maintainer installs the first-party `cargo-tigerstyle` binary and
  runs `cargo tigerstyle check`
- **THEN** tigerstyle resolves its own lint source or packaged artifact
- **AND** the maintainer does not need to set `TIGERSTYLE_PATH`

#### Scenario: Nix-managed nightly still works

- **WHEN** a consumer runs `cargo tigerstyle check` on a nix-managed nightly
  toolchain where `rustup show active-toolchain` would not report the final
  nightly triple directly
- **THEN** tigerstyle still resolves the correct toolchain and runs the check

#### Scenario: New consumer runs tigerstyle without copying a script

- **WHEN** a maintainer onboards a new Rust workspace
- **THEN** they use `cargo tigerstyle check` instead of copying the historical
  shell wrapper

#### Scenario: Explicit scope override beats metadata defaults

- **WHEN** a consumer runs `cargo tigerstyle check -p crate-a`
- **THEN** tigerstyle checks the explicit scope requested on the command line
- **AND** it does not silently replace that scope with the metadata default

#### Scenario: Successful run leaves Cargo.lock unchanged

- **WHEN** `cargo tigerstyle check` finishes successfully for a consumer
  workspace
- **THEN** the consumer workspace `Cargo.lock` matches its pre-run contents

#### Scenario: Warning-only findings stay non-fatal

- **WHEN** `cargo tigerstyle check` finds only warning-level lint diagnostics
- **THEN** the command keeps those diagnostics as warnings
- **AND** it exits `0`

#### Scenario: Error-level findings use lint-failure exit code

- **WHEN** `cargo tigerstyle check` finds an error-level lint diagnostic
- **THEN** it exits `1`

#### Scenario: Invalid metadata uses integration-failure exit code

- **WHEN** `[workspace.metadata.tigerstyle]` is invalid or incomplete
- **THEN** `cargo tigerstyle check` exits `2`

#### Scenario: Unexpected lockfile mutation fails closed

- **WHEN** the check path would otherwise mutate the consumer `Cargo.lock`
- **THEN** tigerstyle restores the original lockfile and exits with an error

### Requirement: Workspace Metadata Owns Runner Configuration

Consumer-specific tigerstyle runner configuration MUST live in
`[workspace.metadata.tigerstyle]` in the consumer workspace `Cargo.toml`.

That metadata MUST define the no-argument check behavior through these keys:

- `default_scope`
- `cargo_check_args`

Tigerstyle MUST read that metadata when `cargo tigerstyle check` runs without
explicit overrides.

When the caller supplies explicit `cargo check` tail args after `--`, tigerstyle
MUST use that explicit tail and MUST NOT merge metadata `cargo_check_args` into
that invocation.

Invalid or incomplete `[workspace.metadata.tigerstyle]` runner metadata MUST
fail with a user-facing configuration error instead of silently falling back to
copied-script behavior.

Consumer-owned `dylint.toml` MUST remain the place for lint tuning such as
thresholds, disabled lints, and lint levels; runner configuration MUST NOT
require script-body edits.

#### Scenario: No-argument check uses workspace metadata

- **WHEN** a consumer runs `cargo tigerstyle check` with no scope flags
- **THEN** tigerstyle uses the default scope and cargo-check args from
  `[workspace.metadata.tigerstyle]`

#### Scenario: Explicit cargo-check args replace metadata defaults

- **WHEN** a consumer runs `cargo tigerstyle check -p crate-a -- --all-targets`
- **THEN** tigerstyle uses the explicit `--all-targets` tail for that run
- **AND** it does not merge metadata `cargo_check_args` into the same command

#### Scenario: Lint tuning stays in dylint.toml

- **WHEN** a consumer changes lint thresholds or disabled lints
- **THEN** they update `dylint.toml`
- **AND** they do not need to edit runner logic in a copied script

### Requirement: First-Class Nix Consumer Integration

Tigerstyle MUST provide a first-class Nix integration path for downstream Nix
consumers that uses the same core engine and workspace metadata contract as
`cargo tigerstyle check`.

That Nix integration MUST:

- expose a flake app for `nix run`
- expose a first-party flake helper for consumer `checks`
- avoid copied consumer shell scripts as the supported path
- allow a consumer flake to wire tigerstyle checks into both `nix run` and
  `nix flake check`
- call the same Rust engine used by `cargo tigerstyle check`

#### Scenario: Nix consumer runs tigerstyle through flake app

- **WHEN** a Nix-based consumer invokes the published tigerstyle flake app
- **THEN** the run uses the same configured scope and policy as
  `cargo tigerstyle check`

#### Scenario: Nix consumer wires tigerstyle into flake checks

- **WHEN** a Nix-based consumer adds tigerstyle to its flake checks
- **THEN** it uses the first-party Nix integration surface instead of a copied
  wrapper script

### Requirement: Repo-Local Entry Points Stay Thin

Tigerstyle MUST treat every published repo-local consumer entry point as a thin
delegation layer over the first-class tigerstyle engine.

Any repo-local tigerstyle entry point published for downstream consumers — such
as `just`, `cargo xtask`, CI helper commands, or generated local wrappers —
MUST be a thin delegation layer over the first-class tigerstyle engine.

Tigerstyle MUST provide a verification fixture, generated helper, or equivalent
reviewable artifact proving that documented helper entry points delegate without
reimplementing policy.

Those entry points MUST NOT reimplement rollout logic such as:

- toolchain detection
- driver bootstrap
- lockfile guarding
- default-scope policy
- metadata parsing

#### Scenario: just target delegates to the canonical command

- **WHEN** a consumer publishes `just tigerstyle`
- **THEN** that target delegates to `cargo tigerstyle check` or the equivalent
  first-party Nix entry point
- **AND** it does not carry a second copy of tigerstyle rollout logic

#### Scenario: CI helper stays policy-thin

- **WHEN** a consumer adds tigerstyle to CI
- **THEN** the CI command delegates to the same first-class engine used locally
- **AND** changes to tigerstyle rollout behavior happen in one implementation
  instead of many repo-local scripts
