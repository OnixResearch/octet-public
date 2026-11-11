# Proof Implementation Gap Specification

## Purpose

Defines the `proof-implementation-gap` capability.

## Requirements

### Requirement: The four specification assumptions are named

r[octet.proof_implementation_gap.assumptions] Octet MUST document the proof-to-implementation failure class with exactly four specification assumptions: Synchronous process assumption, Request-to-response correlation assumption, Atomic persistence of related protocol state, and Formal coverage of the entire protocol, including extensions.

#### Scenario: All four assumptions are named

- GIVEN a reader opens the failure-class documentation
- WHEN the reader reads the assumption list
- THEN it MUST name Synchronous process assumption, Request-to-response correlation assumption, Atomic persistence of related protocol state, and Formal coverage of the entire protocol, including extensions.

#### Scenario: The class is reachable from the repository entry points

- GIVEN a reader starts at the README, the trust-boundary evidence index, the contributor guide, or the law-tested deterministic cores guide
- WHEN the reader follows the documentation links
- THEN the reader MUST reach the failure-class documentation without searching the repository.

### Requirement: The class names three layers and their closing evidence

r[octet.proof_implementation_gap.layers] The failure-class documentation MUST present the class as three layers named interleaving, fault handling, and unverified extensions, and each layer MUST name the system-level testing that closes it.

#### Scenario: Each layer names its closing evidence

- GIVEN a reviewer inspects one layer of the failure class
- WHEN the reviewer reads that layer
- THEN it MUST name the layer, the specification assumption or assumptions at risk in it, and the kind of system-level testing that closes it.

#### Scenario: Layer membership is unambiguous

- GIVEN a defect where the implementation violates one of the four specification assumptions
- WHEN a reviewer classifies the defect
- THEN the reviewer MUST be able to assign it to exactly one of the interleaving, fault handling, or unverified extensions layers.

### Requirement: The claim boundary stays honest

r[octet.proof_implementation_gap.claim_boundary] The failure-class documentation MUST state that a verified model is not a verified implementation, MUST NOT claim that any existing proof is unsound, and MUST NOT claim that the system-level testing which closes the three layers exists in this repository.

#### Scenario: Verified model is distinguished from verified implementation

- GIVEN a reader reads the failure-class documentation
- WHEN the reader looks for the headline claim of the class
- THEN the reader MUST see that a verified model is not a verified implementation.

#### Scenario: No unsoundness claim is made

- GIVEN the failure-class documentation
- WHEN a reviewer inspects its statements about existing proofs
- THEN it MUST NOT assert that any existing proof is unsound.
- AND it MUST state that a proof which states its assumptions remains correct about the model that states them.

#### Scenario: No system-level testing claim is made

- GIVEN the failure-class documentation describes the evidence that closes each layer
- WHEN a reviewer asks whether that evidence exists in this repository today
- THEN the documentation MUST state that this repository does not currently run system-level interleaving, fault-injection, or extension-conformance testing for these layers.

### Requirement: Property-based testing does not close the layers

r[octet.proof_implementation_gap.property_testing_boundary] The property-based testing doctrine MUST state that law and property tests over a pure core do not close the interleaving, fault handling, or unverified extension layers, and MUST point to the failure-class documentation.

#### Scenario: Doctrine points at the failure class

- GIVEN a reader opens the law-tested deterministic cores documentation
- WHEN the reader reads the boundary of the law doctrine
- THEN it MUST state that law and property tests do not close the three layers of the proof-to-implementation gap.
- AND it MUST link to the failure-class documentation.

#### Scenario: A passing law suite is not implementation evidence

- GIVEN a law suite or Hegel property run passes for a pure core
- WHEN the result is reported as implementation assurance
- THEN the report MUST NOT present it as evidence that the deployed implementation is correct.

### Requirement: The class records a reporting shape

r[octet.proof_implementation_gap.reporting] The failure-class documentation MUST state how a defect in the class is reported, including the violated assumption, the layer, the observed versus modeled behavior, and the system-level test that reproduces the defect or the open obligation when none exists.

#### Scenario: A reader can file the defect

- GIVEN a reviewer finds an implementation behavior that violates one of the four specification assumptions
- WHEN the reviewer follows the failure-class reporting guidance
- THEN the reviewer MUST be able to name the violated assumption, the layer, the observed and modeled behavior, and the reproducing system-level test or the open obligation.

#### Scenario: A report does not become a proof verdict

- GIVEN a report names a violated assumption and a layer
- WHEN the report is reviewed
- THEN it MUST be treated as a claim about the implementation boundary.
- AND it MUST NOT be treated as a claim that the corresponding proof is wrong.
