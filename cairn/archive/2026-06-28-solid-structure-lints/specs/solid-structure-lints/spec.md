## ADDED Requirements

### Requirement: fat_trait lint (ISP)

r[octet.solid_structure_lints.fat_trait] Octet MUST ship a `fat_trait` structure lint that flags traits exceeding a configurable method count, defaulting to `warn`.

#### Scenario: Trait over the method budget is flagged

r[octet.solid_structure_lints.fat_trait.over_budget]

- GIVEN a `trait` declares more than `trait_method_max` methods (default 7)
- WHEN the `fat_trait` lint runs
- THEN it MUST emit a `warn`-level diagnostic naming the trait and its method count.
- AND the remediation MUST suggest splitting the trait into cohesive role interfaces.

#### Scenario: Configurable method budget

r[octet.solid_structure_lints.fat_trait.config]

- GIVEN a consumer sets `trait_method_max = 3` in `dylint.toml`
- WHEN the `fat_trait` lint runs
- THEN it MUST use the configured threshold instead of the default 7.

### Requirement: concrete_construction_in_core lint (DIP)

r[octet.solid_structure_lints.concrete_construction_in_core] Octet MUST ship a `concrete_construction_in_core` structure lint that flags external concrete construction inside declared core scopes and MUST no-op when no core scope is declared.

#### Scenario: External construction in core is flagged

r[octet.solid_structure_lints.concrete_construction_in_core.flagged]

- GIVEN a declared core scope contains `<ExternalType>::new()` or `::default()` for a non-domain, non-`std` type
- WHEN the lint runs
- THEN it MUST emit a `warn`-level diagnostic naming the constructed type and the core location.
- AND the remediation MUST suggest injecting the dependency through a port or capability.

#### Scenario: No declared core scope is a no-op

r[octet.solid_structure_lints.concrete_construction_in_core.noop]

- GIVEN no core scope is declared
- WHEN the lint runs
- THEN it MUST emit zero diagnostics.

### Requirement: type_accumulation lint (SRP)

r[octet.solid_structure_lints.type_accumulation] Octet MUST ship a `type_accumulation` structure lint that flags a single type with excessive `impl` blocks or a module with excessive unrelated `pub` items, defaulting to `allow`.

#### Scenario: Excessive impl blocks flagged

r[octet.solid_structure_lints.type_accumulation.impl_blocks]

- GIVEN a single type accumulates more than `impl_blocks_max` (default 4) `impl` blocks
- WHEN the lint runs
- THEN it MUST emit an `allow`-level diagnostic naming the type and its impl-block count.
- AND a module exceeding `module_pub_items_max` (default 12) unrelated `pub` items MUST be flagged separately.

### Requirement: defaulting_trait_method lint (LSP)

r[octet.solid_structure_lints.defaulting_trait_method] Octet MUST ship a `defaulting_trait_method` safety lint that flags trait methods carrying a default body, defaulting to `allow` given high false-positive risk.

#### Scenario: Defaulting trait method flagged

r[octet.solid_structure_lints.defaulting_trait_method.flagged]

- GIVEN a `trait` method item carries a default body
- WHEN the lint runs
- THEN it MUST emit an `allow`-level diagnostic naming the trait and method.
- AND the remediation MUST suggest making the method required or documenting why the default is safe.
- AND a scoped `#[allow(octet::defaulting_trait_method)]` MUST suppress it.
