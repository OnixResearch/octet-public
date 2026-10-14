## Phase 1: Implementation

- [x] [serial] r[octet.accumulating_validation.config_errors] Inventory config, policy, fixture-manifest, proof-manifest, and evidence-manifest validators where independent semantic errors can be accumulated safely.
- [x] [serial] r[octet.accumulating_validation.config_errors] Add typed diagnostic aggregation for the highest-value validators, preserving stable diagnostic ordering and source/field context.
- [x] [serial] r[octet.accumulating_validation.parser_boundaries] Replace selected ad-hoc mini-grammar parsing with named parser functions or small parser modules that return typed structures and diagnostics.
- [x] [serial] r[octet.accumulating_validation.fail_fast_boundary] Preserve fail-fast behavior for syntax errors and for dependent execution, artifact acceptance, evidence promotion, or freshness checks after invalid prerequisites.
- [x] [serial] r[octet.accumulating_validation.typed_diagnostics] Update docs or help text where aggregated diagnostics change review ergonomics.

## Phase 2: Verification

- [x] [serial] r[octet.accumulating_validation.config_errors.multiple_errors] Verify a single invalid config or manifest with multiple independent semantic errors reports all expected diagnostics in stable order.
- [x] [serial] r[octet.accumulating_validation.parser_boundaries.negative_grammar] Verify malformed selectors, scope entries, artifact locators, or policy keys fail with deterministic parser diagnostics rather than substring-derived surprises.
- [x] [serial] r[octet.accumulating_validation.typed_diagnostics.machine_readable] Verify aggregated diagnostics include stable field/path/source context and machine-readable kinds where the validator exposes structured output.
- [x] [serial] r[octet.accumulating_validation.fail_fast_boundary.no_execution] Verify invalid validation results prevent downstream command execution, artifact acceptance, evidence promotion, and freshness success.
- [x] [serial] Run focused config/policy/manifest tests plus Cairn validate/gates before closeout. [depends:cairn-gates]
