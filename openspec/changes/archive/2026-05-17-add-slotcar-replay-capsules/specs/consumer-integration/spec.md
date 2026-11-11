## ADDED Requirements

### Requirement: Slotcar Replay Capsule Capture [r[slotcar-replay-capsule-capture]]

The system MUST support a replay-capsule capture contract for Slotcar lint and fingerprint runs. A capsule MUST record capsule schema version, repo revision or dirty-state summary, Slotcar binary identity, command argv, config hash, selected input file hashes, output artifact hashes, and redacted environment allowlist.

#### Scenario: Capture pins check run identity [r[slotcar-replay-capsule-capture.scenario.check-run-identity]]

- GIVEN `cargo slotcar check` runs with JSON or SARIF output and an artifact directory
- WHEN replay-capsule capture is requested
- THEN the capsule records the command argv, Slotcar binary identity, config hash, selected source hashes, and output artifact hashes
- AND the capsule does not include raw secrets, absolute home paths, local usernames, or unbounded environment dumps

#### Scenario: Capture pins fingerprint run identity [r[slotcar-replay-capsule-capture.scenario.fingerprint-run-identity]]

- GIVEN `cargo slotcar fingerprint --check --review-manifest <path>` runs against selected source files
- WHEN replay-capsule capture is requested
- THEN the capsule records the review manifest hash, selected source hashes, fingerprint output hash, and fingerprint command argv
- AND the capsule keeps review metadata traceable without claiming the fingerprint proves semantic correctness

### Requirement: Slotcar Replay Capsule Replay [r[slotcar-replay-capsule-replay]]

The system MUST replay a captured Slotcar capsule against the current workspace or an explicit checkout and report whether observed inputs and outputs match the capsule. Replay MUST distinguish exact matches, expected-failure matches, and drift.

#### Scenario: Replay accepts exact match [r[slotcar-replay-capsule-replay.scenario.exact-match]]

- GIVEN a replay capsule captured from a clean Slotcar run
- WHEN the same command, config, selected inputs, and output artifacts are replayed unchanged
- THEN replay reports `match`
- AND the report names the capsule hash and every matched artifact hash

#### Scenario: Replay reports drift [r[slotcar-replay-capsule-replay.scenario.drift]]

- GIVEN a replay capsule with pinned source and output hashes
- WHEN a selected input, config, Slotcar binary identity, or output artifact differs
- THEN replay reports `drift`
- AND the report names the mismatched field and expected/observed hash values

#### Scenario: Replay accepts expected failure [r[slotcar-replay-capsule-replay.scenario.expected-failure]]

- GIVEN a replay capsule declares an expected nonzero exit code and expected output hash
- WHEN replay observes the same exit code and output hash
- THEN replay reports `expected_failure_match`
- AND the report does not treat that expected failure as a clean successful lint run
