## ADDED Requirements

### Requirement: Policy-driven crate boundary mechanism

r[octet.crate_boundary_config.mechanism] The architecture checker MUST read core, shell, and adapter crate scopes, forbidden core dependencies, and external domain leakage patterns from a `crate-boundary` mechanism in the Nickel policy registry rather than from hardcoded literals.

#### Scenario: Boundary scopes are declared in policy

r[octet.crate_boundary_config.mechanism.declared]

- GIVEN a consumer declares a `crate-boundary` mechanism with `core_crates`, `shell_crates`, `adapter_crates`, `forbidden_core_deps`, and `external_domain_patterns`
- WHEN the architecture checker runs the FCIS, forbidden-dependency, shell-depends-on-core, and external-leakage checks
- THEN every check MUST use the declared scopes and patterns from policy.
- AND no check MUST match a hardcoded `tigerstyle-core` or `tigerstyle-std` literal when a policy entry is present.

#### Scenario: Malformed boundary config fails closed

r[octet.crate_boundary_config.mechanism.fail_closed]

- GIVEN a `crate-boundary` mechanism omits `core_crates` or declares an empty non-default profile
- WHEN Nickel evaluates the policy contract
- THEN evaluation MUST fail with a deterministic diagnostic before the checker consumes the policy.

### Requirement: Octet-layout default profile equivalence

r[octet.crate_boundary_config.default_profile] The default profile MUST reproduce the exact diagnostics the hardcoded literals produced before this change.

#### Scenario: Default profile is byte-identical to baseline

r[octet.crate_boundary_config.default_profile.baseline]

- GIVEN the `octet-layout` default profile is active against the octet repo sources
- WHEN the architecture checker runs
- THEN the emitted diagnostics MUST be byte-identical to a baseline captured before this change.

### Requirement: Non-silencing fallback

r[octet.crate_boundary_config.fallback] A consumer that declares no `crate-boundary` mechanism MUST receive the octet-layout defaults rather than an empty obligation set.

#### Scenario: Missing mechanism falls back to defaults

r[octet.crate_boundary_config.fallback.defaults]

- GIVEN a consumer declares no `crate-boundary` mechanism
- WHEN the architecture checker resolves boundary obligations
- THEN the resolved obligations MUST equal the octet-layout defaults.
- AND the checks MUST NOT be silently skipped.

### Requirement: Generalized architecture checks

r[octet.crate_boundary_config.checks] The no_std, forbidden-dependency, shell-depends-on-core, and external-domain-leakage checks MUST resolve their target paths and patterns from the policy mechanism.

#### Scenario: Custom-layout consumer is checkable

r[octet.crate_boundary_config.checks.custom_layout]

- GIVEN a consumer with a `domain/` core and `infrastructure/` shell declares a matching `crate-boundary` mechanism
- WHEN the architecture checker runs
- THEN it MUST flag a missing no_std, a forbidden runtime dependency, a shell missing the core dependency, and an external domain leak in the declared core scope.
- AND the same checks MUST pass when the custom layout satisfies the declared obligations.

#### Scenario: Boundary config is receipt-covered

r[octet.crate_boundary_config.artifact]

- GIVEN a resolved `crate-boundary` config is exported
- WHEN the config-receipt freshness check runs
- THEN the exported artifact MUST carry a BLAKE3 digest and MUST fail freshness when the Nickel source drifts.
