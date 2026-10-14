## ADDED Requirements

### Requirement: Optional Hegel property evidence rail
r[octet.dynamic_evidence.hegel_property_rail] Octet MUST expose Hegel RS property-test receipts as an opt-in external evidence rail and MUST NOT require Hegel for starter `cargo octet check`.

#### Scenario: Operator collects Hegel evidence explicitly
r[octet.dynamic_evidence.hegel_property_rail.explicit]
- GIVEN a workspace has produced Hegel property-test output for selected pure-core laws
- WHEN the operator runs the Hegel property evidence rail collection command
- THEN Octet MUST write scoped Hegel property receipts without changing starter check requirements.

#### Scenario: Starter check remains lightweight
r[octet.dynamic_evidence.hegel_property_rail.starter]
- GIVEN a consumer runs starter `cargo octet check`
- WHEN no Hegel rail is requested
- THEN Octet MUST NOT invoke Hegel, require Hegel artifacts, or bootstrap a Hegel backend.

### Requirement: Hegel property receipt identity
r[octet.dynamic_evidence.hegel_property_receipt] Hegel property receipts MUST bind property identity, source scope, linked requirement or policy IDs, Hegel runner identity, deterministic settings, generator identity, status, diagnostics, BLAKE3 artifact identities, and explicit non-claims.

#### Scenario: Complete receipt passes
r[octet.dynamic_evidence.hegel_property_receipt.complete]
- GIVEN a Hegel receipt contains property ID, checked scope, linked IDs, Hegel crate/backend identity, settings, case budget, phases, seed or derandomized mode, generator identity, status, artifact hashes, and non-claims
- WHEN Octet validates the receipt
- THEN validation MUST preserve those fields as deterministic generated-testing evidence metadata.

#### Scenario: Missing or stale identity fails closed
r[octet.dynamic_evidence.hegel_property_receipt.fail_closed]
- GIVEN a Hegel receipt lacks source scope, runner identity, settings, generator identity, artifact hashes, or required non-claims, or records a stale generator hash
- WHEN Octet validates the receipt
- THEN validation MUST fail with deterministic diagnostics naming the rejected field.

#### Scenario: Failing property preserves counterexample
r[octet.dynamic_evidence.hegel_property_receipt.counterexample]
- GIVEN a Hegel property run finds a minimal failing example
- WHEN Octet records the receipt
- THEN the receipt MUST preserve bounded failure diagnostics and a counterexample summary or content hash suitable for review and regression fixture creation.

### Requirement: Hegel evidence boundary
r[octet.dynamic_evidence.hegel_property_boundary] Hegel property receipts MUST remain scoped generated-testing evidence and MUST NOT be promoted to formal proof, exhaustive coverage, whole-program correctness, verifier soundness, or release eligibility.

#### Scenario: Passing Hegel run stays generated-testing evidence
r[octet.dynamic_evidence.hegel_property_boundary.generated_testing]
- GIVEN a Hegel property receipt reports a passing run
- WHEN Octet classifies the receipt
- THEN the evidence tier MUST represent bounded generated-input testing rather than formal proof.

#### Scenario: Promotion attempt fails closed
r[octet.dynamic_evidence.hegel_property_boundary.no_promotion]
- GIVEN a Hegel receipt attempts to claim formal proof, exhaustive correctness, semantic equivalence, requirement satisfaction, release eligibility, or verifier soundness
- WHEN Octet validates the receipt
- THEN validation MUST fail with a deterministic tier or overclaim diagnostic.

#### Scenario: Operator sees non-claims
r[octet.dynamic_evidence.hegel_property_boundary.docs]
- GIVEN an operator reads the Hegel rail guide or receipt summary
- WHEN the supported claim is described
- THEN the output MUST state that Hegel checks bounded generated inputs only and does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety.
