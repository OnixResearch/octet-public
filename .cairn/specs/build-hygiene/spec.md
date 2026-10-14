# Build Hygiene Specification

## Purpose

Defines the `build-hygiene` capability.

## Requirements

### Requirement: Warning-clean owned workspace checks

r[octet.build_hygiene.warning_clean] Octet owned Rust workspace checks SHOULD be warning-clean under the Nix devshell, and release-readiness rails SHOULD treat new owned-code warnings as regressions.

#### Scenario: Workspace check has no owned warnings

r[octet.build_hygiene.warning_clean.workspace]

- GIVEN the workspace is checked under the documented Nix devshell
- WHEN the warning-clean rail runs
- THEN owned Octet packages SHOULD emit no Rust compiler warnings.

#### Scenario: New warning fails the rail

r[octet.build_hygiene.warning_clean.regression]

- GIVEN a change introduces a new owned-code compiler warning
- WHEN the warning-clean rail runs
- THEN the rail SHOULD fail with diagnostics naming the warning source.

### Requirement: Dead-code classification

r[octet.build_hygiene.dead_code_classification] Rust helpers that are only used by tests, generators, or archived compatibility paths MUST be classified as live product code, test-only code, generated-code support, or obsolete code before warnings are suppressed.

#### Scenario: Test-only generator is scoped

r[octet.build_hygiene.dead_code_classification.test_only]

- GIVEN a generator helper is only used from tests
- WHEN warning-clean validation runs
- THEN the helper MUST be scoped to test code or moved to test support rather than left as warning-bearing production code.

#### Scenario: Obsolete helper is removed

r[octet.build_hygiene.dead_code_classification.obsolete]

- GIVEN a helper is not used by product code, tests, generation, or compatibility behavior
- WHEN warning-clean cleanup runs
- THEN the helper MUST be removed instead of hidden behind a broad dead-code allow.

### Requirement: Compatibility binary warning coverage

r[octet.build_hygiene.alias_targets] Warning-clean validation SHOULD cover the canonical `cargo-octet` binary and compatibility binary targets that ship the same engine under legacy command names.

#### Scenario: Alias targets stay clean

r[octet.build_hygiene.alias_targets.clean]

- GIVEN compatibility binaries are built from the same package
- WHEN the warning-clean rail checks the package
- THEN the canonical and compatibility targets SHOULD be warning-clean or share the same explicit, justified exemption.

### Requirement: Pedantic Clippy release-readiness coverage

r[octet.build_hygiene.clippy_pedantic] Octet's release-readiness Clippy seam SHOULD enable `clippy::pedantic` with warnings denied for owned workspace targets, and current pedantic findings SHOULD be fixed or justified narrowly rather than hidden behind broad suppression.

#### Scenario: Pedantic Clippy rail is clean

r[octet.build_hygiene.clippy_pedantic.clean]

- GIVEN the workspace is checked under the documented Nix devshell or flake check
- WHEN the Clippy release-readiness rail runs
- THEN owned Octet packages SHOULD pass with `-D warnings -W clippy::pedantic`.

#### Scenario: Pedantic finding is narrowly handled

r[octet.build_hygiene.clippy_pedantic.narrow]

- GIVEN a pedantic Clippy lint reports an owned-code issue
- WHEN the issue is addressed
- THEN the code SHOULD prefer a mechanical fix or focused refactor.
- AND any suppression SHOULD be narrow and tied to the smallest item where the lint conflicts with the existing contract.

### Requirement: Warning-clean alias target declarations

r[octet.build_hygiene.alias_target_declarations] Octet compatibility binary targets MUST be declared without triggering Cargo duplicate-source target warnings during documented workspace checks.

#### Scenario: Cargo check has no duplicate-target warning

r[octet.build_hygiene.alias_target_declarations.clean]

- GIVEN the workspace is checked under the documented Nix devshell
- WHEN `cargo check` builds owned canonical and compatibility binary targets
- THEN Cargo output MUST NOT contain duplicate-source target warnings such as `found to be present in multiple build targets` for Octet-owned binaries.

#### Scenario: Alias target source files are distinct wrappers

r[octet.build_hygiene.alias_target_declarations.wrappers]

- GIVEN canonical and compatibility binaries share the same command implementation
- WHEN their Cargo targets are declared
- THEN each binary target SHOULD use a distinct tiny wrapper source or an equivalent warning-clean structure that delegates to the shared implementation.

### Requirement: Alias target behavior preservation

r[octet.build_hygiene.alias_target_behavior] Removing duplicate-source target warnings MUST preserve canonical and compatibility binary behavior.

#### Scenario: Version and help remain available

r[octet.build_hygiene.alias_target_behavior.version_help]

- GIVEN a user invokes any shipped canonical or compatibility binary with version or help flags
- WHEN the alias wrapper dispatches to the shared implementation
- THEN the command MUST return the existing version/help behavior and exit code.

#### Scenario: Legacy command still routes to canonical engine

r[octet.build_hygiene.alias_target_behavior.routing]

- GIVEN a user invokes a documented legacy compatibility command
- WHEN the command performs a supported Octet operation
- THEN it MUST route through the same engine as the canonical command unless a separate deprecation contract explicitly changes that behavior.

### Requirement: Duplicate-target warning regression coverage

r[octet.build_hygiene.alias_target_warning_regression] Octet SHOULD include a focused regression check that rejects reintroduction of Cargo duplicate-source target warnings for owned alias binaries.

#### Scenario: Captured warning text fails regression check

r[octet.build_hygiene.alias_target_warning_regression.reject]

- GIVEN a workspace check output contains `found to be present in multiple build targets` for an Octet-owned alias binary
- WHEN the warning-clean regression check runs
- THEN the check SHOULD fail with a diagnostic naming the duplicate-target warning class.
