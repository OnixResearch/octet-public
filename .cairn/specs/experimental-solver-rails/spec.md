# Experimental Solver Rails Specification

## Purpose

Define an optional Z4-inspired solver evidence rail for recorded SMT/CHC/SAT artifacts.

## Requirements

### Requirement: Optional z4-solver rail profile
r[octet.experimental_solver_rails.z4_solver.profile] Octet MUST define an optional `z4-solver` evidence rail profile that records solver identity, version/provenance, input query identity, logic/profile, result status, proof artifact refs, model or unsat-core refs, timeout or unknown reason, validation policy, diagnostics, verification role, and non-claims.

#### Scenario: Solver receipt validates as recorded evidence
r[octet.experimental_solver_rails.z4_solver.positive]
- GIVEN a collector provides a solver receipt with matching solver identity, query hash, logic profile, result status, artifact refs, diagnostics, and non-claims
- WHEN Octet validates the z4-solver rail receipt
- THEN validation MUST accept the receipt as optional recorded evidence.

### Requirement: Unknown remains non-proof evidence
r[octet.experimental_solver_rails.z4_solver.unknown] Octet MUST treat `unknown`, timeout, unsupported logic, and bounded exhaustion outcomes as non-proof diagnostic evidence.

#### Scenario: Unknown does not satisfy proof rail
- GIVEN a z4-solver receipt reports `unknown` or timeout for a query
- WHEN Octet validates the receipt
- THEN validation MAY preserve diagnostics but MUST NOT count the receipt as a discharged proof obligation.

### Requirement: Solver rail rejects stale or overclaiming artifacts
r[octet.experimental_solver_rails.z4_solver.validation] Octet MUST reject z4-solver receipts that are stale, substituted, malformed, unsupported, or overclaiming.

#### Scenario: Invalid solver receipt fails closed
r[octet.experimental_solver_rails.z4_solver.negative]
- GIVEN a solver receipt has a stale query hash, unsupported logic promoted as pass, mismatched solver identity, missing proof ref for a proof claim, stale proof artifact, role promotion, or broad replacement claim
- WHEN rail validation runs
- THEN validation MUST fail with deterministic diagnostics and MUST NOT emit proof-card promotion.

### Requirement: Solver execution is shell-owned
r[octet.experimental_solver_rails.z4_solver.shell] Octet MUST keep solver execution, process spawning, filesystem reads, timeout enforcement, raw log capture, and presentation outside pure validation cores.

#### Scenario: Collector wraps external execution
- GIVEN a user explicitly collects z4-solver evidence
- WHEN the collector invokes a solver or reads solver artifacts
- THEN shell code MAY perform those effects, but pure validation MUST operate only on parsed in-memory receipt data.

### Requirement: Z4 rail preserves Octet proof boundaries
r[octet.experimental_solver_rails.z4_solver.final_validation] Octet MUST state that passing z4-solver rail validation proves only receipt structure, identity linkage, role conformance, and non-claim preservation unless a later checked proof rail narrows the claim.

#### Scenario: Valid receipt does not replace existing proof rails
- GIVEN a z4-solver receipt validates
- WHEN Octet reports evidence
- THEN the report MUST NOT claim Verus replacement, Kani replacement, Z3 compatibility, solver soundness, solver completeness, arbitrary SMT-LIB compatibility, source-code correctness, whole-program safety, or release eligibility.
