## Phase 1: Implementation

- [x] [serial] r[octet.type_driven_domain_modeling.domain_identities] Inventory raw lint names, group names, diagnostic IDs, evidence family IDs, artifact locators, config keys, and threshold/count values that cross non-trivial core or module boundaries.
- [x] [serial] r[octet.type_driven_domain_modeling.domain_identities] Introduce typed identity wrappers for the highest-risk inventory entries while preserving existing public strings at serialization and CLI boundaries.
- [x] [serial] r[octet.type_driven_domain_modeling.algebraic_states] Replace selected boolean/string-tag state combinations with enums that encode mutually exclusive states directly.
- [x] [serial] r[octet.type_driven_domain_modeling.refined_values] Add refined constructors for non-empty, canonical, sorted/deduplicated, path-normalized, and threshold/count inputs needed by the migrated seams.
- [x] [serial] r[octet.type_driven_domain_modeling.incremental_migration] Keep compatibility adapters at Dylint config, source-attribute, emitted evidence, SARIF, and CLI output boundaries.

## Phase 2: Verification

- [x] [serial] r[octet.type_driven_domain_modeling.domain_identities.raw_boundary] Verify migrated pure cores no longer accept raw string/path/numeric identities for the inventoried seams.
- [x] [serial] r[octet.type_driven_domain_modeling.algebraic_states.invalid_combinations] Verify invalid state combinations are unrepresentable or rejected by constructors before reaching core logic.
- [x] [serial] r[octet.type_driven_domain_modeling.refined_values.negative_inputs] Verify empty collections, malformed identities, non-canonical paths, duplicates, and invalid thresholds fail with deterministic diagnostics.
- [x] [serial] r[octet.type_driven_domain_modeling.incremental_migration.compatibility] Verify existing config keys, source attrs, emitted evidence fields, CLI text, and downstream fixture outputs stay compatible unless explicitly migrated.
- [x] [serial] Run focused config/diagnostic/runner tests plus Cairn validate and gates for this change before closeout. [depends:cairn-gates]
