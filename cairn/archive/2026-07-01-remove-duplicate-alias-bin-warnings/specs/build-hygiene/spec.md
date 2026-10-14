## ADDED Requirements

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
