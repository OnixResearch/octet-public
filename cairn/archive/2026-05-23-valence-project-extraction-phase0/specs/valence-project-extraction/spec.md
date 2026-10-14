# Valence Project Extraction Specification

## Purpose

Define Phase 0 requirements for preparing Valence to become a standalone project while preserving Octet's current bounded evidence claims and avoiding premature code movement.

## Requirements

### Requirement: Phase 0 extraction inventory
r[octet.valence_project_extraction.phase0.inventory] The system MUST record a reviewed Phase 0 inventory of candidate Valence-owned surfaces, Octet-retained surfaces, shared handoff surfaces, and current verification entrypoints before any code or repository cutover occurs.

#### Scenario: Inventory records current surfaces
r[octet.valence_project_extraction.phase0.inventory.current]
- GIVEN the current Octet repository contains Valence-oriented code, fixtures, docs, Cairn specs, and Nix gates
- WHEN the Phase 0 extraction package is reviewed
- THEN it provides an inventory artifact that names the observed path families and verification entrypoints.

### Requirement: Extraction boundary remains planning-only
r[octet.valence_project_extraction.phase0.boundary] The system MUST classify extraction surfaces without moving code, renaming crates, changing package identity, creating a standalone repository, or cutting a release during Phase 0.

#### Scenario: Boundary avoids cutover side effects
r[octet.valence_project_extraction.phase0.boundary.planning_only]
- GIVEN Phase 0 is an extraction-readiness planning package
- WHEN the package is validated
- THEN it does not require code movement, crate renames, repository creation, tag movement, or release publication.

### Requirement: Handoff gates are explicit
r[octet.valence_project_extraction.phase0.gates] The system MUST define the named Nix and Cairn gates that a future Valence extraction must preserve before any cutover can be accepted.

#### Scenario: Gate contract names release and aggregate rails
r[octet.valence_project_extraction.phase0.gates.named]
- GIVEN the current Octet repository exposes Valence evidence release and aggregate gates
- WHEN the Phase 0 handoff contract is inspected
- THEN it names the release evidence gate, aggregate object evidence gate, Cairn validation gate, and current evidence-index checker invocation or an explicit compatibility successor.

### Requirement: Extraction plan preserves bounded claims
r[octet.valence_project_extraction.phase0.non_claims] The system MUST preserve the existing bounded non-claim language when describing Valence extraction readiness.

#### Scenario: Non-claims stay explicit
r[octet.valence_project_extraction.phase0.non_claims.explicit]
- GIVEN the current Valence evidence is selected-corpus and selected-fixture replay evidence
- WHEN Phase 0 readiness is described
- THEN it does not claim correctness, semantic equivalence, exhaustive Rust coverage, general HIR/THIR/MIR extraction, public unattended production readiness, cross-machine determinism, or cross-toolchain determinism.
