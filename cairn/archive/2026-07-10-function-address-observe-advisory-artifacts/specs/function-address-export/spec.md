# Function Address Export Specification

## Purpose

Make observe-mode function-address export usable as the first gradual-adoption step without a pre-existing baseline.

## Requirements

### Requirement: Observe mode emits artifacts with advisory debt
r[octet.function_address_export.observe_artifact_valid.core] Octet MUST emit a function-address artifact in observe mode when Cargo checking succeeds, even when functions lack reviewed markers or baseline entries.

#### Scenario: Unbaselined observe writes report
r[octet.function_address_export.observe_artifact_valid.positive]
- GIVEN a Rust workspace has functions and no function-address baseline
- WHEN `cargo octet check --function-addresses=observe` runs successfully
- THEN Octet MUST write `function-addresses.json` and MUST NOT fail because the missing-review findings are malformed.

### Requirement: Observe findings use the accepted output domain
r[octet.function_address_export.observe_artifact_valid.findings] Octet MUST encode observe-mode function-address findings using a severity, group, path, and identity shape accepted by the runner output validator.

#### Scenario: Advisory finding validates
r[octet.function_address_export.observe_artifact_valid.validation]
- GIVEN observe mode reports missing reviewed function-address coverage
- WHEN structured output and artifact-bundle validation run
- THEN every function-address finding MUST pass the output-domain validator and the artifact bundle MUST remain consumable.

### Requirement: Strict modes fail closed on bad baselines
r[octet.function_address_export.observe_artifact_valid.negative] Octet MUST keep warn and deny mode diagnostics for malformed, stale, or overclaiming reviewed baselines.

#### Scenario: Malformed baseline remains diagnostic
r[octet.function_address_export.observe_artifact_valid.docs]
- GIVEN a reviewed function-address baseline contains malformed BLAKE3, stale source identity, or forbidden claim text
- WHEN warn or deny mode checks the baseline
- THEN Octet MUST report deterministic diagnostics without weakening the function-address non-claim boundary.
