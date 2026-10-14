## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

Verified-mode policies MUST support an explicit command-execution path that runs relevant declared proof commands and treats the resulting command output as the proof receipt for the current check run, while preserving the default read-only artifact-backed receipt check.

#### Scenario: Opt-in proof command execution

- GIVEN a selected verified policy with a relevant proof rail command and source root
- WHEN `tigerstyle-standards check` is run with proof command execution enabled
- THEN the checker MUST run the declared command from the checked source root and evaluate the emitted command receipt for the configured pass signal.

#### Scenario: Default read-only proof receipt check

- GIVEN a selected verified policy with a relevant proof rail and committed proof receipt
- WHEN `tigerstyle-standards check` is run without proof command execution
- THEN the checker MUST evaluate the checked proof receipt without running the declared command.

#### Scenario: Failing proof command blocks verified policy

- GIVEN a selected verified policy whose relevant proof command exits without emitting the configured pass signal
- WHEN proof command execution is enabled
- THEN the checker MUST emit a blocking verified-proof-rail diagnostic for the policy.
