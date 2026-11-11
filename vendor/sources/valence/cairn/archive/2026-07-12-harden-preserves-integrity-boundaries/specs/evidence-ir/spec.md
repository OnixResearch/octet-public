# Evidence IR Specification

## Purpose

Hardens Preserves evidence identity and replaces prose-based claim authority with typed claim-policy boundaries.

## Requirements

### Requirement: Preserves hash material is structurally derived
r[valence.evidence_ir.preserves_integrity.canonical_hash] Valence MUST derive Preserves evidence hash material by parsing the envelope, removing only schema-declared top-level self-hash fields, canonically encoding the remaining typed value, and hashing those canonical bytes with BLAKE3.

#### Scenario: Canonical envelope identity is stable
r[valence.evidence_ir.preserves_integrity.fixtures.positive]
- GIVEN equivalent typed Preserves envelopes with different non-canonical renderings
- WHEN corrected-profile identity is computed
- THEN the canonical packed bytes and BLAKE3 identity MUST match.

#### Scenario: Payload field-name text remains hash-bearing
r[valence.evidence_ir.preserves_integrity.fixtures.negative]
- GIVEN payload content contains `receipt_hash` or `preserves_content_hash` as a key or string value
- WHEN that payload content changes
- THEN the canonical hash material MUST change and validation MUST reject reuse of the prior digest.

#### Scenario: Malformed structure fails closed
- GIVEN an envelope cannot be parsed, has duplicate or misplaced self-hash fields, or violates the declared schema
- WHEN identity validation runs
- THEN validation MUST fail with deterministic structural diagnostics and MUST NOT fall back to rendered-line filtering.

### Requirement: Corrected hash profiles are versioned
r[valence.evidence_ir.preserves_integrity.profile_version] Valence MUST expose a distinct corrected hash-profile identity and MUST reject a receipt whose declared profile does not match the algorithm used to derive its identity.

#### Scenario: Legacy digest cannot claim corrected profile
- GIVEN a receipt carries a digest derived by the vulnerable rendered-line profile
- WHEN it declares the corrected profile
- THEN validation MUST fail with a profile-and-digest mismatch diagnostic.

### Requirement: Legacy migration is explicit and byte-bound
r[valence.evidence_ir.preserves_integrity.migration] Valence MUST migrate a legacy Preserves evidence identity only through an explicit adapter that validates the original envelope bytes and emits a receipt binding the legacy identity, corrected identity, migration profile, and non-claims.

#### Scenario: Missing original bytes block migration
- GIVEN a legacy receipt identity without its original envelope bytes
- WHEN migration is requested
- THEN migration MUST fail closed and MUST NOT synthesize a corrected identity from metadata alone.

### Requirement: Evidence claim authority is typed
r[valence.evidence_ir.typed_claim_boundary.contract] Valence MUST represent supported claim class, verification role, prohibited claim capabilities, and required boundary identifiers as typed policy values rather than granting authority through prose matching.

#### Scenario: Bounded identity claim passes
r[valence.evidence_ir.typed_claim_boundary.fixtures.positive]
- GIVEN an artifact declares an allowed identity/linkage claim class, compatible verification role, required boundary identifiers, and no prohibited capabilities
- WHEN typed claim validation runs
- THEN validation MUST pass while preserving the bounded claim.

#### Scenario: Unknown typed value fails closed
r[valence.evidence_ir.typed_claim_boundary.fixtures.negative]
- GIVEN an artifact declares an unknown claim class, role, boundary identifier, or capability
- WHEN typed claim validation runs
- THEN validation MUST fail with a deterministic unsupported-value diagnostic.

### Requirement: Contradictory overclaims are rejected
r[valence.evidence_ir.typed_claim_boundary.validation] Valence MUST reject records whose typed fields or human-readable metadata assert a capability stronger than the supported typed claim, even when required non-claim phrases are also present.

#### Scenario: Contradictory non-claim text cannot pass
- GIVEN an artifact includes the required identity-only boundary text and also asserts correctness, semantic equivalence, authority, verifier soundness, or release eligibility
- WHEN claim validation runs
- THEN validation MUST fail with a deterministic prohibited-capability diagnostic.

#### Scenario: Prose cannot grant authority
- GIVEN display text claims a stronger capability than the typed claim class allows
- WHEN downstream policy evaluates the artifact
- THEN the display text MUST NOT promote the artifact's verification role or supported claim.

### Requirement: Integrity migration is documented
r[valence.evidence_ir.preserves_integrity.docs] Valence documentation MUST identify the vulnerable profile as legacy, explain corrected-profile selection and migration, and state that passing validation proves canonical identity and typed claim-policy conformance only.

#### Scenario: Downstream acceptance is explicit
- GIVEN a downstream producer or consumer reads the migration guidance
- WHEN it selects accepted evidence profiles
- THEN it can distinguish rejected legacy identities, explicitly migrated identities, and corrected-profile identities without inferring behavioral correctness.

### Requirement: Preserves integrity verification rail
r[valence.evidence_ir.preserves_integrity.final_validation] The change MUST include positive and negative regression evidence for canonicalization, collision resistance at the application profile boundary, typed claims, and migration behavior.

#### Scenario: Repository verification covers the flaw
- GIVEN the implementation and fixtures are complete
- WHEN focused tests and repository validation run
- THEN benign envelopes MUST pass, collision and contradictory-claim fixtures MUST fail, and migration receipts MUST bind original bytes and both identities.

### Requirement: Flake-check CI is checked in
r[valence.evidence_ir.preserves_integrity.flake_check_ci] Valence MUST include a checked-in CI workflow for this remediation whose verification command is `nix flake check`.

#### Scenario: CI uses the scoped verification rail
- GIVEN a change is evaluated by checked-in CI
- WHEN the remediation workflow runs
- THEN it MUST execute `nix flake check` and MUST NOT require a separate expanded CI command matrix in this change.
