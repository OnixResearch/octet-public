## ADDED Requirements

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
