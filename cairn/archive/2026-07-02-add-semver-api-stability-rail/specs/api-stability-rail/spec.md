## ADDED Requirements

### Requirement: SemVer API-stability rail

r[octet.api_stability.semver_rail] Octet SHOULD provide an API-stability rail backed by `cargo-semver-checks` for release candidates that expose Rust library APIs.

#### Scenario: Compatible API passes

r[octet.api_stability.semver_rail.compatible]

- GIVEN a checked package has a valid baseline and no SemVer-breaking findings for the selected feature and target scope
- WHEN the API-stability rail runs
- THEN Octet SHOULD report success.
- AND the report SHOULD identify the checked package, baseline source, feature set, target triple, and `cargo-semver-checks` version.

#### Scenario: Breaking API fails closed

r[octet.api_stability.semver_rail.breaking]

- GIVEN `cargo-semver-checks` reports a denied SemVer finding for the selected release intent
- WHEN Octet evaluates the API-stability rail
- THEN Octet MUST fail the rail with diagnostics preserving the SemVer finding name, affected item, and required update level when provided.

### Requirement: API baseline declaration

r[octet.api_stability.baseline] API-stability execution MUST require an explicit or workspace-declared baseline before it can be treated as release evidence.

#### Scenario: Baseline source is explicit

r[octet.api_stability.baseline.explicit]

- GIVEN an operator configures an API-stability run
- WHEN Octet resolves the baseline
- THEN the baseline MUST identify one of a registry version, VCS revision, local source root, or saved rustdoc JSON artifact.
- AND the receipt MUST record the resolved baseline identity.

#### Scenario: Missing baseline cannot pass

r[octet.api_stability.baseline.missing]

- GIVEN no baseline is configured or passed on the command line
- WHEN the API-stability rail is requested for release evidence
- THEN Octet MUST fail closed with a diagnostic naming the missing baseline field.

### Requirement: API-stability receipts

r[octet.api_stability.receipt] API-stability runs SHOULD emit deterministic `policy_check` evidence that names the scope and known non-claims.

#### Scenario: Receipt records checked scope

r[octet.api_stability.receipt.scope]

- GIVEN the API-stability rail completes
- WHEN Octet writes its receipt
- THEN the receipt SHOULD include schema version, checker ID, command argv, package list, baseline identity, current identity, feature flags, target triple, findings, result status, and bounded log identity.

#### Scenario: Receipt preserves API-review boundary

r[octet.api_stability.receipt.boundary]

- GIVEN an API-stability receipt reports success
- WHEN the receipt is rendered or verified
- THEN it MUST NOT claim behavioral compatibility, business-domain compatibility, or exhaustive SemVer coverage beyond the selected checker and scope.
