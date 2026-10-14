# Accumulating Validation Specification Delta

## ADDED Requirements

### Requirement: Independent config errors are accumulated

r[octet.accumulating_validation.config_errors] Octet validators for human-authored or human-reviewed config, policy, fixture manifests, proof manifests, and evidence manifests SHOULD accumulate independent semantic diagnostics after syntax parsing succeeds.

#### Scenario: Multiple semantic errors are reported together

r[octet.accumulating_validation.config_errors.multiple_errors]

- GIVEN a parseable config or manifest contains multiple independent semantic errors such as unknown lint names, invalid levels, duplicate identities, unsupported evidence families, missing required fields, or stale generated-artifact references
- WHEN validation runs
- THEN validation SHOULD report all independent errors in deterministic order.
- AND it MUST NOT produce an accepted validated value.

### Requirement: Non-trivial grammars have parser boundaries

r[octet.accumulating_validation.parser_boundaries] Octet SHOULD parse non-trivial selector, scope, policy, artifact-locator, and manifest mini-grammars through named parser functions or focused parser modules rather than scattered ad-hoc string splitting.

#### Scenario: Malformed grammar fails deterministically

r[octet.accumulating_validation.parser_boundaries.negative_grammar]

- GIVEN a selector, scope entry, policy key, artifact locator, or manifest field uses malformed grammar
- WHEN parsing runs
- THEN the parser SHOULD return a typed diagnostic naming the rejected grammar element.
- AND downstream semantic validation SHOULD NOT infer missing structure from substring matches.

### Requirement: Aggregated diagnostics are typed and located

r[octet.accumulating_validation.typed_diagnostics] Aggregated validation diagnostics SHOULD carry stable error kinds plus field, source, path, or item context sufficient for deterministic tests and reviewer action.

#### Scenario: Structured output keeps diagnostics machine-readable

r[octet.accumulating_validation.typed_diagnostics.machine_readable]

- GIVEN a validator exposes structured output, receipt fields, or machine-readable diagnostics
- WHEN multiple validation errors are emitted
- THEN each diagnostic SHOULD include a stable kind and field/path/source context.
- AND human text SHOULD remain deterministic across repeated runs for the same input.

### Requirement: Invalid validation fails closed before dependent stages

r[octet.accumulating_validation.fail_fast_boundary] Diagnostic accumulation MUST NOT permit dependent command execution, artifact acceptance, evidence promotion, or freshness success after required validation has failed.

#### Scenario: Invalid aggregate blocks downstream execution

r[octet.accumulating_validation.fail_fast_boundary.no_execution]

- GIVEN validation accumulates one or more errors for a required config, policy, manifest, proof declaration, or evidence declaration
- WHEN the caller would otherwise execute tools, accept generated artifacts, promote evidence, or mark freshness successful
- THEN the caller MUST fail closed before the dependent stage.
- AND the diagnostic output SHOULD preserve the accumulated validation errors.
