## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

The system MUST require proof-rail evidence when a policy declares `current = "verified"` or targets verified enforcement for critical primitives.

Verified enforcement MUST name the proof backend, source/proof mapping, expected command, accepted proof artifact shape, and trust-boundary accounting. Verified enforcement MUST also provide artifact-backed checker diagnostics when declared local proof evidence is absent or does not contain the configured pass signal.

#### Scenario: Proof rail obligations are exported [r[verified-enforcement-rails.scenario.proof-rail-obligations-exported]]

- GIVEN a policy declares verified enforcement with proof rail metadata
- WHEN generated policy artifacts are exported
- THEN the generated check config summarizes proof rail backends, commands, source paths, proof paths, pass signals, and trust boundaries
- AND the artifact-backed checker can consume those proof rail fields without reading Nickel source directly

#### Scenario: Compliant proof rail receipt passes [r[verified-enforcement-rails.scenario.compliant-proof-rail-passes]]

- GIVEN checked source includes the declared source path and a declared proof receipt path
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the verification family
- THEN the checker exits zero when the proof receipt contains the configured pass signal
- AND the evaluated policy metadata classifies the failure type as `verified-proof-rail`

#### Scenario: Missing proof pass signal fails [r[verified-enforcement-rails.scenario.missing-proof-pass-signal-fails]]

- GIVEN checked source includes a proof receipt path without the configured pass signal
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the verification family
- THEN the checker exits nonzero with a verified-proof diagnostic naming the missing pass signal
- AND the remediation tells the developer to rerun the proof command and save a passing receipt

#### Scenario: Focused implementation checks pass [r[verified-enforcement-rails.scenario.focused-implementation-checks-pass]]

- GIVEN the proof-rail checker, artifacts, registry, fixtures, and OpenSpec package have been updated
- WHEN the focused Rust, Nickel/Nix, and OpenSpec validation commands run
- THEN every command exits successfully
- AND the saved transcript names the exact commands used

#### Scenario: Proof rail gate does not overclaim semantic proof execution [r[verified-enforcement-rails.scenario.no-semantic-proof-overclaim]]

- GIVEN the checker validates proof receipt shape and pass-signal presence
- WHEN reviewers inspect the design and evidence
- THEN the change states that `tigerstyle-standards check` does not execute arbitrary proof commands
- AND semantic proof execution remains the responsibility of the saved deterministic evidence command outside the pure checker
