# Evidence Stack Specification

## Purpose

Defines the `evidence-stack` capability.

## Requirements

### Requirement: Evidence stack layers

r[octet.evidence_stack.layers] Octet documentation MUST present an evidence stack with discipline, static edge, policy layer, and evidence chain layers.

#### Scenario: Evidence stack is complete

r[octet.evidence_stack.layers.complete]

- GIVEN consumer-facing documentation describes the Octet evidence stack
- WHEN a reader inspects the stack
- THEN it MUST name the discipline, static edge, policy layer, and evidence chain layers.
- AND each layer MUST map to at least one concrete Octet surface, command, document, or artifact.

### Requirement: Evidence stack is not a proof-stack overclaim

r[octet.evidence_stack.no_proof_stack] Octet documentation MUST NOT describe the evidence stack as an unqualified proof stack.

#### Scenario: Proof language is scoped

r[octet.evidence_stack.no_proof_stack.scoped]

- GIVEN documentation uses proof or verification language
- WHEN the language describes the evidence stack
- THEN the text MUST distinguish lint evidence, policy validation, transcript assertions, and formal proof artifacts.
- AND unqualified `proof stack` wording MUST fail validation.
