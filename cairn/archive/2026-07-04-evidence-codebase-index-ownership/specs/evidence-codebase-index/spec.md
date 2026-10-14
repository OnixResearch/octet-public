# Evidence Codebase Index Specification Delta

## ADDED Requirements

### Requirement: Evidence codebase index ownership is explicit

r[octet.evidence_codebase_index.ownership] Octet MUST identify the evidence-codebase index as an Octet-owned review/indexing capability unless a future reviewed change parameterizes product identity or moves the core to a different crate.

#### Scenario: Output boundary names current ownership

r[octet.evidence_codebase_index.ownership.output_boundary]

- GIVEN generated JSON, query output, HTML, and human documentation describe the evidence-codebase index
- WHEN a reviewer inspects product identity and release ownership language
- THEN the output MUST identify the current capability as Octet evidence indexing.
- AND it MUST NOT imply standalone Valence release ownership, package publication, repository cutover, or transfer of Octet evidence release responsibilities.

#### Scenario: Valence-hosted pure core boundary is documented

r[octet.evidence_codebase_index.ownership.valence_hosted]

- GIVEN the pure evidence-codebase index implementation is hosted under `valence-core`
- WHEN `valence-core` package and API docs describe the module
- THEN they SHOULD state that Octet retains CLI parsing, filesystem traversal, artifact discovery, output writing, Nix gate ownership, release bundle ownership, and non-claim boundary enforcement.

### Requirement: Evidence codebase index core is decomposed by role

r[octet.evidence_codebase_index.module_decomposition] The system SHOULD decompose the evidence-codebase index pure core into reviewable model, ingestion, query, HTML rendering, and boundary helpers while preserving existing public API behavior.

#### Scenario: Public API remains stable after decomposition

r[octet.evidence_codebase_index.module_decomposition.public_api]

- GIVEN callers use `build_evidence_codebase_index`, `query_evidence_index`, `render_evidence_index_html`, and `output_has_required_boundary`
- WHEN the implementation is decomposed behind those functions
- THEN the callers SHOULD compile unchanged.
- AND the decomposed helpers SHOULD remain pure deterministic transforms over in-memory inputs.

### Requirement: Evidence codebase index compatibility is preserved

r[octet.evidence_codebase_index.compatibility] The system MUST preserve the evidence-codebase index JSON schema, deterministic sorting, strict/advisory issue semantics, query linkage behavior, and HTML non-claim boundary unless a future reviewed migration updates schema and fixtures.

#### Scenario: Outputs remain byte-stable

r[octet.evidence_codebase_index.compatibility.byte_stable]

- GIVEN the same ordered supported evidence artifact fixture set and normalization version
- WHEN the decomposed evidence-codebase index core builds JSON, query output, and HTML twice
- THEN records, linked query results, HTML boundary text, and top-level index hash MUST remain deterministic and byte-stable.

#### Scenario: Ownership and claim overclaims fail closed

r[octet.evidence_codebase_index.compatibility.overclaim_negative]

- GIVEN generated JSON, query output, HTML, or docs are tampered to claim behavioral correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, formal proof for non-proof tiers, or standalone Valence release ownership
- WHEN boundary validation scans the output
- THEN validation MUST fail closed with deterministic diagnostics naming the unsupported claim class.
