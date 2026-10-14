# Mirai Semantic Pilot Specification

## Purpose

Defines the `mirai-semantic-pilot` capability.

## Requirements

### Requirement: MIRAI pilot provider identity is exact

r[octet.mirai_semantic_pilot.provider]

The pilot MUST bind an exact MIRAI revision, compatible rustc toolchain, target, features, command argv, environment allowlist, Nix derivation, closure graph, source scope, limits, and non-claims.

#### Scenario: Pinned provider preflight passes

- GIVEN every provider, toolchain, target, command, environment, source, and limit identity matches the reviewed pilot policy
- WHEN pilot preflight runs
- THEN the provider MAY execute the selected corpus
- AND the preflight receipt MUST bind every exact identity

#### Scenario: Ambient or drifted provider fails

- GIVEN the MIRAI revision, rustc, target, feature set, argv, environment, derivation, closure, or source identity differs from policy
- WHEN preflight runs
- THEN the pilot MUST fail closed before semantic evidence collection

### Requirement: MIRAI pilot policy is typed and bounded

r[octet.mirai_semantic_pilot.profile]

Octet MUST define typed Nickel policy for the selected corpus, supported observation vocabulary, resource limits, comparison outcomes, promotion criteria, decision states, and non-claims.

#### Scenario: Complete policy exports deterministically

- GIVEN the reviewed pilot policy contains every required identity, limit, criterion, outcome, and non-claim
- WHEN Nickel validation and export run
- THEN generated runtime policy MUST match the checked-in artifact byte-for-byte

#### Scenario: Missing criterion fails

- GIVEN pilot policy omits a resource limit, comparison outcome, promotion criterion, blocked state, or non-claim
- WHEN policy validation runs
- THEN validation MUST fail with a deterministic field diagnostic

### Requirement: Pilot corpus has explicit properties and failures

r[octet.mirai_semantic_pilot.corpus]

The pilot MUST use a finite selected Rust corpus with explicit positive properties, negative properties, unsupported-feature labels, source identities, and expected diagnostic classes.

#### Scenario: Supported fixture has a reviewable oracle

- GIVEN a selected fixture represents a safe path, reachable panic, checked precondition, call edge, or effect-relevant operation
- WHEN the fixture enters the pilot
- THEN it MUST name the expected observation and exact source scope

#### Scenario: Unsupported fixture remains visible

- GIVEN a selected fixture uses a construct outside the pinned MIRAI or pilot normalization contract
- WHEN the pilot evaluates it
- THEN the result MUST be `unsupported` or `blocked` with a named reason
- AND the fixture MUST NOT be silently excluded from reported coverage

### Requirement: MIRAI observations are recorded through a stable boundary

r[octet.mirai_semantic_pilot.evidence]

Octet MUST normalize only stable machine-readable MIRAI observations and MUST record an exact blocked result when no maintainable output contract exists.

#### Scenario: Supported observation normalizes

- GIVEN the pinned provider emits a supported machine-readable observation for a selected fixture
- WHEN the pure normalization core runs twice
- THEN both normalized artifacts and issue ordering MUST match
- AND the receipt MUST bind raw output, normalized output, provider, source, status, limits, and BLAKE3 identities

#### Scenario: Human diagnostic scraping is required

- GIVEN a required pilot fact is available only through unstable or ambiguous human-formatted diagnostics
- WHEN output-contract evaluation runs
- THEN the pilot MUST record a blocked output-contract result
- AND it MUST NOT claim semantic evidence for that fact

### Requirement: Cross-analyzer comparison preserves authority

r[octet.mirai_semantic_pilot.comparison]

Octet MAY compare aligned MIRAI, Charon, and source-token facts, but each result MUST preserve parent identities and use only declared comparison outcomes.

#### Scenario: Comparable facts agree

- GIVEN all parents bind the same selected source identity and report compatible facts
- WHEN comparison runs
- THEN the row MAY report `agree` with every parent identity
- AND agreement MUST NOT promote the evidence tier or claim semantic equivalence

#### Scenario: Facts disagree or cannot align

- GIVEN facts disagree, source identities differ, one fact is missing, or alignment is ambiguous
- WHEN comparison runs
- THEN the row MUST report `disagree` or `unknown` without rewriting any parent

### Requirement: MIRAI adoption decision is measured and deterministic

r[octet.mirai_semantic_pilot.decision]

The pilot MUST emit an `adopt`, `defer`, `reject`, or `blocked` decision bound to declared signal, reproducibility, stability, runtime, compatibility, and maintenance criteria.

#### Scenario: Adoption criteria pass

- GIVEN the provider and corpus complete and every required promotion criterion passes
- WHEN decision evaluation runs
- THEN the receipt MAY report `adopt`
- AND a separate Cairn change MUST define any production evidence rail or default integration

#### Scenario: Criterion fails or authority is missing

- GIVEN one required criterion fails, a provider cannot build, output is unstable, or a required measurement is unavailable
- WHEN decision evaluation runs
- THEN the receipt MUST report `defer`, `reject`, or `blocked` with exact failed criteria
- AND no production MIRAI dependency or rail MUST be added by this pilot

### Requirement: MIRAI pilot remains independent from default paths

r[octet.mirai_semantic_pilot.boundary]

The pilot MUST NOT change starter checks, strict profiles, effect-closure v1, Charon, Aeneas, Kani, Verus, or release eligibility behavior.

#### Scenario: Ordinary Octet paths remain unchanged

- GIVEN an operator does not select the MIRAI pilot
- WHEN existing Octet commands run
- THEN they MUST NOT require MIRAI artifacts, binaries, policy, or toolchain inputs

#### Scenario: Pilot overclaims

- GIVEN a pilot artifact claims MIRAI soundness, complete Rust support, whole-program correctness, semantic equivalence, or release eligibility
- WHEN artifact validation runs
- THEN validation MUST fail with a claim-boundary diagnostic

### Requirement: MIRAI pilot verification covers positive and negative paths

r[octet.mirai_semantic_pilot.verification]

The pilot MUST include positive and negative fixtures for provider identity, policy, corpus oracles, output contracts, comparisons, decisions, limits, drift, default independence, and claim boundaries.

#### Scenario: Pilot matrix produces a bounded result

- GIVEN all declared positive, negative, unsupported, drift, timeout, disagreement, and overclaim fixtures
- WHEN focused pilot validation runs
- THEN every supported positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
- AND the pilot MUST emit a decision or exact blocked result
