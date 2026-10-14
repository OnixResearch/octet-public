## Contract

- [x] [serial] Specify the `zero_alloc_attr_contract` lint: recognized marker set, allocation classifier scope, direct-source-shape-only disclaimer, and `warn` default level. r[octet.purity_effect_lints.zero_alloc_attr_contract]
- [x] [serial] Specify non-emitting shapes for stack locals, array init, primitive returns, and explicit `allow` with justification. r[octet.purity_effect_lints.zero_alloc_attr_contract.allows_non_allocating]
- [x] [serial] Specify the auditable escape-hatch contract: any `allow` of `zero_alloc_attr_contract` MUST be justified by the existing `unjustified_allow` lint. r[octet.purity_effect_lints.zero_alloc_attr_contract.auditable_escape_hatch]
- [x] [serial] Specify the shared-classifier invariant: `zero_alloc_attr_contract` and `allocation_in_loop` reuse one allocation classifier. r[octet.purity_effect_lints.zero_alloc_attr_contract.shared_classifier]

## Implementation

- [x] [serial] Implement the `EffectClass::Allocation` and `PureViolation::Allocation` variants and a pure allocation-classifier function in `src/purity/effect_lints.rs`, reusing the `allocation_in_loop.rs` allocator table. r[octet.purity_effect_lints.zero_alloc_attr_contract]
- [x] [serial] Implement the `zero_alloc_attr_contract` late-pass shell and recognized-marker detection (`#[tigerstyle::zero_alloc]`, `#[octet::zero_alloc]`, `#[zero_alloc]`). r[octet.purity_effect_lints.zero_alloc_attr_contract]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.zero_alloc_attr_contract.catalog]

## Verification

- [x] [serial] Verify marked allocating function is flagged. r[octet.purity_effect_lints.zero_alloc_attr_contract.flags_allocation]
- [x] [parallel] Verify non-allocating or explicitly-allowed function stays clean. r[octet.purity_effect_lints.zero_alloc_attr_contract.allows_non_allocating]
- [x] [parallel] Verify auditable escape hatch rejects silent allow. r[octet.purity_effect_lints.zero_alloc_attr_contract.auditable_escape_hatch]
- [x] [serial] Verify allocation classifier is shared with the loop lint. r[octet.purity_effect_lints.zero_alloc_attr_contract.shared_classifier]
- [x] [serial] Verify allocation classifier unit tests. r[octet.purity_effect_lints.zero_alloc_attr_contract.flags_allocation]
- [x] [serial] Update fixture manifests and run cargo test --lib, Cairn validation, and Cairn gates. r[octet.purity_effect_lints.zero_alloc_attr_contract.catalog]

## Phase 3: Human/oracle checkpoints

- [x] [serial] Lint reference documents zero_alloc_attr_contract exactly once. r[octet.purity_effect_lints.zero_alloc_attr_contract.catalog]

## Change Status
- Status: finished
- Open tasks: none

## Verification Coverage

- Scenario: Marked allocating function is flagged -> requires UI test runner (blocked by dylint alias conflict)
- Scenario: Non-allocating or explicitly-allowed function stays clean -> requires UI test runner
- Scenario: Auditable escape hatch rejects silent allow -> requires UI test runner
- Scenario: Allocation classifier is shared with the loop lint -> PASS
- Scenario: Catalog and fixture coverage are fresh -> PASS