# Proof Artifact Library Specification

## Purpose

Ingest Trellis proof evidence IR as reference evidence unless Octet reruns and validates its own proof receipt.

## Requirements

### Requirement: Trellis proof IR intake is recorded-only by default
r[octet.proof_artifact_library.trellis_proof_ir_intake.recorded_only] Octet MUST treat imported Trellis proof evidence IR as recorded-only reference evidence unless an Octet-owned proof rerun receipt satisfies promotion checks.

#### Scenario: Reference proof does not count as Octet proof
r[octet.proof_artifact_library.trellis_proof_ir_intake.positive]
- GIVEN Octet imports a valid Trellis proof evidence-chain envelope
- WHEN no Octet-owned verifier rerun receipt is provided
- THEN Octet MUST record the artifact as reference evidence and MUST NOT count it as a promoted formal proof.

### Requirement: Intake validates source, IR, verifier, and assumptions
r[octet.proof_artifact_library.trellis_proof_ir_intake.contract] Octet MUST validate Trellis proof evidence identity shape, source envelope linkage, proof IR fields, verifier receipt metadata, assumptions, property IDs, and non-claim boundaries.

#### Scenario: Stale or incomplete proof evidence is rejected
r[octet.proof_artifact_library.trellis_proof_ir_intake.negative]
- GIVEN Trellis proof evidence has stale source, missing verifier identity, missing assumptions, malformed property IDs, or weakened non-claims
- WHEN Octet intake validation runs
- THEN Octet MUST reject or mark the evidence invalid with deterministic diagnostics.

### Requirement: Formal-proof promotion requires Octet rerun identity
r[octet.proof_artifact_library.trellis_proof_ir_intake.promotion] Octet MUST require matching source, verifier/toolchain identity, solver identity, command identity, assumptions, property scope, and passed result status before promoting Trellis-derived proof evidence to Octet formal-proof role.

#### Scenario: Unauthorized role promotion fails closed
- GIVEN imported Trellis evidence declares formal-proof status without an Octet rerun receipt
- WHEN Octet validates the intake record
- THEN validation MUST fail with an unauthorized-promotion diagnostic.

### Requirement: Assumption compatibility is explicit
r[octet.proof_artifact_library.trellis_proof_ir_intake.assumptions] Octet MUST compare Trellis proof assumptions and trusted boundaries against Octet policy before any promotion.

#### Scenario: Stack smoke preserves the proof boundary
r[octet.proof_artifact_library.trellis_proof_ir_intake.validation]
- GIVEN Trellis proof IR flows through the stack smoke
- WHEN Octet consumes it as reference evidence
- THEN the evidence chain MUST preserve recorded-only boundaries unless Octet reruns the proof.
