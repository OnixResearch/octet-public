## ADDED Requirements

### Requirement: Zero-alloc attribute contract lint

r[octet.purity_effect_lints.zero_alloc_attr_contract] Octet MUST provide a `zero_alloc_attr_contract` lint that flags direct heap-allocation sites in functions marked with Octet's recognized zero-allocation marker and guides maintainers toward stack-local construction, pre-allocated buffers, or shell-owned allocation.

#### Scenario: Marked allocating function is flagged

r[octet.purity_effect_lints.zero_alloc_attr_contract.flags_allocation]

- GIVEN a Rust function is marked with the recognized zero-allocation marker (`#[tigerstyle::zero_alloc]`, `#[octet::zero_alloc]`, or `#[zero_alloc]`)
- WHEN its body directly constructs a heap-allocating value such as `Vec::new()`, `Box::new(..)`, `String::from(..)`, `.to_string()`, `format!(..)`, or another allocator constructor covered by Octet's shared allocation classifier
- THEN `zero_alloc_attr_contract` MUST emit a diagnostic at the allocation site.
- AND the diagnostic MUST state that the marker enforces direct source-shape checks on allocation sites rather than transitive whole-call-tree non-allocation proof.

#### Scenario: Non-allocating or explicitly-allowed function stays clean

r[octet.purity_effect_lints.zero_alloc_attr_contract.allows_non_allocating]

- GIVEN a marked function computes from stack locals, array initialization, primitive returns, or struct construction of stack-only fields
- WHEN the lint runs
- THEN `zero_alloc_attr_contract` MUST NOT emit for that function.
- AND a function that legitimately allocates MAY silence the lint with an explicit `allow` paired with a justification.

#### Scenario: Auditable escape hatch rejects silent allow

r[octet.purity_effect_lints.zero_alloc_attr_contract.auditable_escape_hatch]

- GIVEN a function uses a bare `#[allow(tigerstyle::zero_alloc_attr_contract)]` or `#[allow(zero_alloc_attr_contract)]` without a justification comment
- WHEN the `unjustified_allow` lint runs
- THEN `unjustified_allow` MUST emit for that allow.
- AND Octet MUST NOT provide a silent `assume`-style attribute that disables `zero_alloc_attr_contract` without a visible diff artifact.

#### Scenario: Allocation classifier is shared with the loop lint

r[octet.purity_effect_lints.zero_alloc_attr_contract.shared_classifier]

- GIVEN `zero_alloc_attr_contract` and `allocation_in_loop` both classify allocator constructors
- WHEN the shared allocation classifier is updated to recognize a new allocator type
- THEN both lints MUST classify that type consistently and their UI fixtures MUST be updated together.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.zero_alloc_attr_contract.catalog]

- GIVEN `zero_alloc_attr_contract` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, marker syntax docs, and downstream rollout guidance MUST include the lint identity and default level.
