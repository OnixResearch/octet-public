# Gate Posture Specification

## Purpose

Defines the `gate-posture` capability.

## Requirements

### Requirement: Default-deny gate posture documentation
r[octet.default_deny_gate] Octet MUST provide documentation defining a default-on gate posture where the checked path runs before the build, no unimplemented bypass flag is claimed and CI must always run the gate, exit codes are stable for automation, and the gate is byte-deterministic.

#### Scenario: Posture doc exists
r[octet.default_deny_gate.doc_exists]
- GIVEN the Octet documentation tree
- WHEN a consumer looks for the default-deny posture
- THEN `docs/default-deny-gate-posture.md` exists and defines default-on, no-hidden-bypass, stable-exit-code, and byte-deterministic guarantees

### Requirement: Checked-build wrapper template
r[octet.default_deny_gate.alias] Octet MUST provide a copyable wrapper template that makes the checked path the default-on build command. Cargo aliases cannot chain commands, so the template is a shell script rather than a `.cargo/config.toml` alias.

#### Scenario: Wrapper template exists
r[octet.default_deny_gate.alias.exists]
- GIVEN the Octet templates directory
- WHEN a consumer looks for the checked-build wrapper template
- THEN `templates/octet-check.sh` exists, runs `cargo octet check`, and runs `cargo check` with the caller's arguments only after the gate exits `0`

### Requirement: CI gate template
r[octet.default_deny_gate.ci] Octet MUST provide a copyable CI step template that runs the gate with SARIF output and artifact verification.

#### Scenario: CI template exists
r[octet.default_deny_gate.ci.exists]
- GIVEN the Octet templates directory
- WHEN a consumer looks for the CI gate template
- THEN `templates/ci-gate.yml` exists and runs `cargo octet check` with `--output-format sarif --artifact-dir` followed by `cargo octet artifact verify`
