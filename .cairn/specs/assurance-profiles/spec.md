# Assurance Profiles Specification

## Purpose

Defines the `assurance-profiles` capability.

## Requirements

### Requirement: Typed linked profile [r[octet.assurance_profiles.typed_contract]]

Octet MUST provide a closed Nickel assurance-profile contract with a stable schema version, profile ID, summary, subject scope, assumptions, trusted computing base, mechanisms, required evidence, and explicit non-claims.

Each mechanism MUST name its owner, assurance plane, protected property, assumption references, trusted-base references, evidence references, and mechanism-specific non-claims.

#### Scenario: Complete profile exports

- GIVEN a profile supplies every required field with unique and resolved IDs
- WHEN Nickel typechecking and export run
- THEN the profile MUST export deterministically.
- AND the output MUST retain every declared owner, scope, property, assumption, trusted-base element, evidence requirement, validation command, and non-claim.

### Requirement: Cross-field failures fail closed [r[octet.assurance_profiles.fail_closed]]

The assurance-profile contract MUST reject empty required arrays, duplicate IDs, unresolved references, unreferenced declarations, unsupported assurance planes, unknown fields, and malformed subject scope.

#### Scenario: Malformed or disconnected profile fails closed

- GIVEN a fixture has an empty non-claim set, duplicate ID, unresolved reference, unreferenced declaration, unsupported plane, unknown field, or malformed scope
- WHEN Nickel validation runs
- THEN validation MUST fail.
- AND the invalid profile MUST NOT receive accepted identity evidence.

### Requirement: Deterministic profile identity [r[octet.assurance_profiles.identity]]

Each exported profile MUST have a deterministic Nickel Export manifest that binds the exact authored source, contract dependency, generated JSON bytes, evaluator identity, generation command, byte length, and BLAKE3 digests.

#### Scenario: Profile identity is deterministic

- GIVEN identical source, contract, evaluator identity, and output bytes
- WHEN profile generation and manifest construction run twice
- THEN the generated JSON and manifest MUST be byte-identical.
- AND the manifest output BLAKE3 MUST identify the selected profile artifact.

#### Scenario: Profile drift changes or invalidates identity

- GIVEN the source, contract dependency, output bytes, or evaluator identity changes
- WHEN freshness validation runs against the prior manifest
- THEN validation MUST fail or produce a different identity.
- AND the diagnostic MUST name the stale identity class.

### Requirement: OnixOS lifecycle pilot [r[octet.assurance_profiles.onixos_pilot]]

Octet MUST provide an `onixos-lifecycle-proof-v1` pilot that maps the active OnixOS lifecycle proof chain across construction, semantic-design, verification, effect-control, and provenance-readiness planes.

#### Scenario: OnixOS pilot preserves all assurance planes and gaps

- GIVEN the pilot profile is exported
- WHEN an operator inspects its mechanisms and required evidence
- THEN every assurance plane MUST have at least one mechanism.
- AND the profile MUST require Octet conformance, canonical Kamacite FSM identity, Rust transition conformance, Verus proof, Kani proof, Valence linkage, and Cairn proof-chain evidence.
- AND the profile MUST state that these requirements do not report evidence completion.

### Requirement: Consumable package [r[octet.assurance_profiles.distribution]]

Octet MUST expose the pilot profile, contract, export spec, generated JSON, and BLAKE3 manifest through a first-class Nix package.

#### Scenario: Packaged profile is consumable without runtime Nickel

- GIVEN a downstream consumer selects the package at an immutable Octet revision
- WHEN it reads the generated JSON and manifest
- THEN it MUST NOT need to evaluate Nickel for ordinary profile consumption.
- AND source regeneration MUST remain an explicit build or freshness-validation action.

### Requirement: Compatibility and claim boundaries [r[octet.assurance_profiles.boundaries]]

Assurance profiles MUST remain declarations of required mechanisms and evidence. They MUST NOT claim evidence success, automatic claim strengthening, whole-program correctness, hardware isolation, physical behavior, tool soundness, or release eligibility.

#### Scenario: Ordinary Octet checks remain unchanged

- GIVEN a consumer runs `cargo octet check` without selecting an assurance-profile package
- WHEN Octet prepares and executes the check
- THEN existing runner, strict-profile, and evidence behavior MUST remain unchanged.
- AND ordinary checks MUST NOT evaluate assurance-profile Nickel source.

#### Scenario: Profile language preserves non-claims

- GIVEN an operator reads the pilot profile, package, or documentation
- WHEN the supported claim is described
- THEN the text MUST describe typed declaration conformance and exact identity only.
- AND it MUST NOT describe the profile as a universal security score, zero-trust system, complete dependency proof, or release decision.
