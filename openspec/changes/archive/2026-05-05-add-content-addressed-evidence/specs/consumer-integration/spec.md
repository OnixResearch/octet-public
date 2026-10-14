## ADDED Requirements

### Requirement: Content Addressed Finding Identity [r[content-addressed-finding-identity]]

The system MUST support optional content-addressed identity fields for Tigerstyle findings, including finding fingerprint, source hash, snippet hash, config hash, and tool revision.

#### Scenario: Finding hash fields appear on request [r[content-addressed-finding-identity-scenario-finding-hash-fields-appear-on-request]]

- GIVEN JSON output is requested with content-addressed fields enabled
- WHEN a lint finding is emitted
- THEN the JSON record includes stable hash fields without absolute paths or timestamps

#### Scenario: Line moves do not define identity [r[content-addressed-finding-identity-scenario-line-moves-do-not-define-identity]]

- GIVEN a finding moves line number while its normalized snippet and lint context remain stable
- WHEN fingerprints are compared
- THEN the finding identity remains attributable without relying on the old line number

### Requirement: Content Addressed Evidence Headers [r[content-addressed-evidence-headers]]

The system MUST define evidence transcript headers that pin command, config, tool revision, fixture hashes, and repository revision without recording secrets or unstable local environment values.

#### Scenario: Evidence header is deterministic [r[content-addressed-evidence-headers-scenario-evidence-header-is-deterministic]]

- GIVEN the same command, config, fixtures, tool revision, and repo revision
- WHEN an evidence header is regenerated
- THEN the header hashes are identical across repeated runs
