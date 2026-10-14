# Unsafe Trust Boundary Specification

## Purpose

Defines the `unsafe-trust-boundary` capability.

## Requirements

### Requirement: Unsafe trusted-boundary records

r[octet.unsafe_trust_boundary.record] Octet documentation SHOULD define an unsafe/trusted-boundary record for new or changed unsafe, raw-pointer, FFI, or lifetime-erasing seams. The record SHOULD include boundary name, public entry point, unsafe operation, protected invariant, caller obligation, safe wrapper or exception rationale, evidence tier, dynamic evidence selection, reviewer action, and explicit non-claim.

#### Scenario: Record fields are defined

r[octet.unsafe_trust_boundary.record.fields]

- GIVEN a reviewer opens the canonical unsafe/trusted-boundary guidance
- WHEN they inspect the record definition
- THEN it SHOULD define boundary name, public entry point, unsafe operation, protected invariant, caller obligation, safe wrapper or exception rationale, evidence tier, dynamic evidence selection, reviewer action, and explicit non-claim fields.
- AND it SHOULD state that the record is review evidence, not proof by itself.

#### Scenario: Record is searchable

r[octet.unsafe_trust_boundary.record.searchable]

- GIVEN a repository contains a documented unsafe/trusted-boundary record
- WHEN a reviewer searches for unsafe-boundary records
- THEN the record SHOULD use stable labels or headings that make the boundary easy to find during audit.

### Requirement: Public unsafe API wrapper or exception

r[octet.unsafe_trust_boundary.public_api] Octet guidance SHOULD state that public Rust `unsafe fn` APIs should normally be replaced by safe public wrappers around small private unsafe helpers, or documented as explicit exceptions with caller obligations and reviewer action.

#### Scenario: Safe wrapper path is documented

r[octet.unsafe_trust_boundary.public_api.safe_wrapper]

- GIVEN a `public_unsafe_api` finding appears in a production API
- WHEN the maintainer chooses the normal remediation path
- THEN guidance SHOULD direct them to expose a safe public wrapper, keep the unsafe helper private or narrowly scoped, and state the invariant enforced by the wrapper.

#### Scenario: Exception path is documented

r[octet.unsafe_trust_boundary.public_api.exception]

- GIVEN a low-level crate intentionally exposes public unsafe API
- WHEN the maintainer does not hide it behind a safe wrapper
- THEN guidance SHOULD require an exception rationale, inherited caller obligations, evidence tier, reviewer action, and explicit non-claim.

### Requirement: FFI trusted import boundaries

r[octet.unsafe_trust_boundary.ffi] Octet guidance SHOULD treat FFI wrappers, raw handles, erased lifetimes, foreign allocation, and external ownership transfer as trusted import boundaries requiring stable handle strategy, error mapping, ownership and lifetime notes, and review evidence.

#### Scenario: FFI wrapper prompt names ownership and errors

r[octet.unsafe_trust_boundary.ffi.prompts]

- GIVEN a change adds or modifies an FFI wrapper
- WHEN a reviewer applies the unsafe/trusted-boundary guidance
- THEN the guidance SHOULD ask how handles remain stable across the boundary.
- AND it SHOULD ask how foreign error codes map into Rust errors.
- AND it SHOULD ask who owns allocation and deallocation.
- AND it SHOULD ask which lifetimes or aliasing assumptions are erased or reconstructed.

### Requirement: Selected dynamic evidence for high-risk boundaries

r[octet.unsafe_trust_boundary.dynamic_evidence] Octet guidance SHOULD require high-risk unsafe or FFI boundary changes to select an appropriate dynamic evidence profile or record why dynamic evidence is not applicable for the scoped change.

#### Scenario: High-risk boundary selects evidence profile

r[octet.unsafe_trust_boundary.dynamic_evidence.selected]

- GIVEN a change touches unsafe code, raw pointers, C ABI, lifetime erasure, foreign allocation, or external ownership transfer
- WHEN the maintainer marks the change high risk
- THEN the review record SHOULD name the selected dynamic evidence profile, command or fixture, saved transcript or artifact, and acceptance signal.

#### Scenario: Starter checks stay lightweight

r[octet.unsafe_trust_boundary.dynamic_evidence.lightweight_default]

- GIVEN a consumer runs starter `cargo octet check`
- WHEN no unsafe/FFI dynamic profile is explicitly selected
- THEN Octet guidance MUST NOT require heavyweight dynamic tools for the starter static check path.

### Requirement: Unsafe boundary claim boundaries

r[octet.unsafe_trust_boundary.boundaries] Unsafe/trusted-boundary guidance MUST NOT claim that Octet verifies unsafe internals, provides Salt-style `@trusted` semantics, or makes unsafe code safe by documentation alone.

#### Scenario: Verified-unsafe overclaim fails validation

r[octet.unsafe_trust_boundary.boundaries.overclaim_check]

- GIVEN documentation validation scans unsafe/trusted-boundary guidance
- WHEN it encounters a claim that Octet verifies unsafe internals, provides Salt-style `@trusted`, or makes unsafe code safe by documentation alone
- THEN validation MUST fail unless the claim is explicitly negated or scoped to a separate accepted formal-proof receipt.

#### Scenario: Formal proof remains separate

r[octet.unsafe_trust_boundary.boundaries.formal_proof_separate]

- GIVEN an unsafe-boundary record cites a formal proof receipt
- WHEN documentation explains the claim
- THEN it MUST scope the proof to the declared property, verifier identity, source and proof binding, receipt, and artifact identity.
- AND it MUST keep lint, dynamic evidence, transcript, and manual review evidence in their own evidence tiers.
