# Gate Posture Specification

## Purpose

Defines the `gate-posture` capability.

## Requirements

### Requirement: Default-deny gate posture documentation
r[octet.default_deny_gate] Octet MUST provide documentation defining a default-on gate posture where the checked path runs before the build, bypass flags are explicitly labeled as CI-must-not, exit codes are stable for automation, and the gate is byte-deterministic.

#### Scenario: Posture doc exists
r[octet.default_deny_gate.doc_exists]
- GIVEN the Octet documentation tree
- WHEN a consumer looks for the default-deny posture
- THEN `docs/default-deny-gate-posture.md` exists and defines default-on, bypass-labeled, stable-exit-code, and byte-deterministic guarantees

### Requirement: Cargo alias template
r[octet.default_deny_gate.alias] Octet MUST provide a copyable cargo alias template that makes the checked path the default-on build command.

#### Scenario: Alias template exists
r[octet.default_deny_gate.alias.exists]
- GIVEN the Octet templates directory
- WHEN a consumer looks for the cargo alias template
- THEN `templates/cargo-alias.toml` exists and defines `octet-check` as `!cargo octet check -- && cargo check`

### Requirement: CI gate template
r[octet.default_deny_gate.ci] Octet MUST provide a copyable CI step template that runs the gate with SARIF output and artifact verification.

#### Scenario: CI template exists
r[octet.default_deny_gate.ci.exists]
- GIVEN the Octet templates directory
- WHEN a consumer looks for the CI gate template
- THEN `templates/ci-gate.yml` exists and runs `cargo octet check` with `--output-format sarif --artifact-dir` followed by `cargo octet artifact verify`
