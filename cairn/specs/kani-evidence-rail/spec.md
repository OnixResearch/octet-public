# Kani Evidence Rail Specification

## Purpose

Defines the `kani-evidence-rail` capability.

## Requirements

### Requirement: Optional Kani evidence collection
r[octet.kani_evidence_rail.optional_collection] Octet MUST expose Kani model-check receipts as an opt-in external evidence rail and MUST NOT require Kani for starter `cargo octet check`.

#### Scenario: Operator collects Kani evidence explicitly
r[octet.kani_evidence_rail.optional_collection.explicit]
- GIVEN a workspace has produced Kani verification output for selected harnesses
- WHEN the operator runs the Kani evidence rail collection command
- THEN Octet MUST write a scoped Kani receipt without changing starter check requirements.

#### Scenario: Starter check remains lightweight
r[octet.kani_evidence_rail.optional_collection.starter]
- GIVEN a consumer runs starter `cargo octet check`
- WHEN no Kani rail is requested
- THEN Octet MUST NOT invoke Kani or require Kani artifacts.

### Requirement: Kani receipt identity
r[octet.kani_evidence_rail.receipt_identity] Kani receipts MUST bind harness identity, source scope, normalized command identity, Kani/rustc/CBMC/solver identity, unwind policy, verdict, diagnostic summary, input hash, and receipt hash.

#### Scenario: Complete identity passes
r[octet.kani_evidence_rail.receipt_identity.complete]
- GIVEN a Kani receipt contains all required identity and hash fields
- WHEN Octet validates the receipt
- THEN validation MUST preserve the fields as deterministic evidence metadata.

#### Scenario: Missing identity fails closed
r[octet.kani_evidence_rail.receipt_identity.missing]
- GIVEN a Kani receipt lacks harness, toolchain, solver, command, or hash identity
- WHEN Octet validates the receipt
- THEN validation MUST fail with a deterministic missing-identity diagnostic.

### Requirement: Kani evidence classification
r[octet.kani_evidence_rail.classification] Octet MUST classify Kani evidence by bounded harnesses, passing unwinding assertions, contract verification, verified stubs, plain stubs, and failed checks before promotion to any evidence tier.

#### Scenario: Contract proof is scoped property evidence
r[octet.kani_evidence_rail.classification.contract]
- GIVEN a Kani `proof_for_contract` receipt passes for a specific function contract and source hash
- WHEN Octet classifies the receipt
- THEN Octet MAY mark the exact contract property as scoped model-check evidence.

#### Scenario: Plain stub remains a boundary
r[octet.kani_evidence_rail.classification.plain_stub]
- GIVEN a Kani harness uses a plain stub without a linked passing verified contract
- WHEN Octet classifies the receipt
- THEN Octet MUST record the stub as a boundary assumption rather than property evidence.

### Requirement: Kani collection failure modes
r[octet.kani_evidence_rail.collection_failure_modes] Octet MUST fail closed for failed Kani verdicts, failed unwinding assertions presented as unbounded evidence, stale input hashes, missing non-claims, unsupported solver metadata, and unverified-stub promotion attempts.

#### Scenario: Failed verdict blocks passing evidence
r[octet.kani_evidence_rail.collection_failure_modes.failed_verdict]
- GIVEN a Kani receipt reports a failed check
- WHEN Octet validates it for a passing evidence tier
- THEN validation MUST reject the receipt and preserve the failing diagnostic.

#### Scenario: Unwinding failure blocks unbounded claim
r[octet.kani_evidence_rail.collection_failure_modes.unwind]
- GIVEN a bounded harness has a failed or missing unwinding assertion status
- WHEN Octet validates an unbounded claim from that harness
- THEN validation MUST fail with a deterministic unwind-status diagnostic.

### Requirement: Kani fixture coverage
r[octet.kani_evidence_rail.fixture_coverage] Octet MUST include positive and negative fixtures for the Kani evidence rail.

#### Scenario: Positive fixtures cover accepted evidence
r[octet.kani_evidence_rail.fixture_coverage.positive]
- GIVEN fixtures for a passing bounded harness, a passing contract harness, and a verified-stub caller
- WHEN the fixture suite runs
- THEN each fixture MUST produce deterministic accepted Kani evidence receipts.

#### Scenario: Negative fixtures cover broken evidence
r[octet.kani_evidence_rail.fixture_coverage.negative]
- GIVEN fixtures with missing identity, failing verdicts, unwind failures, stale hashes, missing non-claims, or plain-stub promotion
- WHEN the fixture suite runs
- THEN each fixture MUST fail closed with deterministic diagnostics.

### Requirement: Kani boundary documentation
r[octet.kani_evidence_rail.boundary_documentation] Octet documentation MUST state that Kani evidence is scoped model-check evidence and does not prove whole-program correctness, verifier soundness, unmodeled aliasing, concurrency, async behavior, or FFI behavior.

#### Scenario: Operator sees non-claims
r[octet.kani_evidence_rail.boundary_documentation.visible]
- GIVEN an operator reads the Kani rail guide
- WHEN the supported claim is described
- THEN the documentation MUST list the scoped claim and the explicit non-claims.
