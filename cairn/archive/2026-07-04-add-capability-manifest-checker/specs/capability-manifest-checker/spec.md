# Capability Manifest Checker Specification

## Purpose

Defines the `capability-manifest-checker` capability: a Rust-compatible, policy-owned checker that makes effect authority explicit across core, shell, adapter, test, and remote-execution scopes.

## Requirements

### Requirement: Capability manifest schema

r[octet.capability_manifest_checker.manifest] Octet SHOULD provide a typed capability manifest mechanism that declares scope classes, allowed capability families, injected capability traits or tokens, shell interpreters, deterministic handlers, and strict or advisory enforcement mode.

#### Scenario: Valid manifest exports deterministic runtime policy

r[octet.capability_manifest_checker.manifest.valid]

- GIVEN a Nickel-authored capability manifest declares supported scope classes, capability families, injected traits or tokens, shell interpreters, and deterministic handlers
- WHEN policy export and artifact freshness validation run
- THEN the manifest SHOULD export deterministic runtime JSON with a BLAKE3 identity and registry coverage.
- AND the exported policy SHOULD preserve declared strict or advisory enforcement mode.

#### Scenario: Invalid manifest fails closed

r[octet.capability_manifest_checker.manifest.invalid]

- GIVEN a capability manifest omits required fields, names an unknown capability family, names an unknown strict scope class, or declares an empty strict core scope profile
- WHEN policy validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field.

### Requirement: Scope resolution

r[octet.capability_manifest_checker.scope_resolution] Octet SHOULD resolve repo-relative source paths or crates to declared capability scope classes before applying capability obligations.

#### Scenario: Scope resolution is deterministic

r[octet.capability_manifest_checker.scope_resolution.deterministic]

- GIVEN the same manifest and same repo-relative source path inventory
- WHEN scope resolution runs twice
- THEN each selected source path or crate SHOULD resolve to the same scope class and manifest profile.
- AND ambiguous strict matches MUST fail closed rather than choosing an arbitrary scope.

### Requirement: Effect observation matching

r[octet.capability_manifest_checker.effect_matching] Octet SHOULD compare observed direct platform effects and declared injected capabilities against the capability manifest.

#### Scenario: Undeclared direct effect is flagged

r[octet.capability_manifest_checker.effect_matching.direct_effect]

- GIVEN a source file resolves to a strict core scope that does not allow direct filesystem, network, time, randomness, process, environment, secrets, unsafe-code, global-state, concurrency, or observability effects
- WHEN checker observations find a direct platform effect in that source file
- THEN the checker SHOULD emit a deterministic diagnostic naming the effect family, source scope, and required shell or capability boundary.

#### Scenario: Declared injected capability is accepted

r[octet.capability_manifest_checker.effect_matching.injected]

- GIVEN a strict core scope declares a capability trait or authority token for a specific effect family
- WHEN core code accepts that declared trait or token as an explicit input and does not call the concrete platform API directly
- THEN the checker SHOULD accept the injected capability usage.
- AND the report SHOULD link the capability to its declared shell interpreter when one is provided.

### Requirement: Deterministic handler evidence

r[octet.capability_manifest_checker.handlers] Capability reports SHOULD identify deterministic test handlers for declared capabilities when the manifest requires handler evidence.

#### Scenario: Handler evidence is covered

r[octet.capability_manifest_checker.handlers.covered]

- GIVEN a capability manifest requires deterministic handler evidence for a capability family
- WHEN the checker evaluates the declared test fixture or transcript
- THEN the report SHOULD name the handler artifact, replay command, checked scope, success-path coverage, failure-path coverage, and BLAKE3 artifact identity.

#### Scenario: Required handler evidence is missing

r[octet.capability_manifest_checker.handlers.missing]

- GIVEN a strict manifest requires deterministic handler evidence for a capability family
- WHEN no valid handler artifact or replay command is declared
- THEN the checker MUST fail closed with a diagnostic naming the missing handler obligation.

### Requirement: Capability report boundaries

r[octet.capability_manifest_checker.report] Capability reports MUST preserve Octet's validation boundary and MUST NOT describe the checker as a Rust effect system or whole-program purity proof.

#### Scenario: Report avoids effect-system overclaims

r[octet.capability_manifest_checker.boundaries.no_effect_system]

- GIVEN a human, JSON, or documentation report describes capability checking
- WHEN claim-boundary validation scans the report
- THEN it MUST state that the checker validates declared policy and observed source shapes only.
- AND it MUST NOT claim transitive effect inference, complete runtime-effect absence, whole-program purity, or new Rust ability syntax.
