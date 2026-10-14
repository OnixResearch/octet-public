## ADDED Requirements

### Requirement: Fixture Backed Standards Examples [r[fixture-backed-standards-examples]]

The system MUST require standards code examples that claim checker behavior to link to fixture-backed evidence or an explicitly marked prose-only exemption.

#### Scenario: Example links to fixture [r[fixture-backed-standards-examples-scenario-example-links-to-fixture]]

- GIVEN a standards page shows a bad or good Rust example for a checker behavior
- WHEN example verification runs
- THEN the example maps to a fixture path, expected output path, and content hashes

#### Scenario: Prose-only example is explicit [r[fixture-backed-standards-examples-scenario-prose-only-example-is-explicit]]

- GIVEN an example is illustrative and not executable
- WHEN documentation verification runs
- THEN the example is marked prose-only with a rationale rather than silently counted as verified

### Requirement: Live Example Verification Command [r[live-example-verification-command]]

The system MUST provide or document a local-first command that validates example manifests against checked-in fixtures and expected outputs.

#### Scenario: Changed fixture invalidates manifest [r[live-example-verification-command-scenario-changed-fixture-invalidates-manifest]]

- GIVEN a fixture body changes without manifest hash update
- WHEN live example verification runs
- THEN the command fails and names the stale example entry
