# Evidence Codebase Index Specification

## Purpose

Defines the `evidence-codebase-index` capability: a local, deterministic, content-addressed index over Octet evidence artifacts for review, query, and static browsing.

## Requirements

### Requirement: Evidence index model

r[octet.evidence_codebase_index.model] Octet MUST define a versioned evidence index model that records indexable artifact family, stable record ID, artifact locator, BLAKE3 content identity where deterministic bytes exist, source IDs, evidence tier, parent links, and explicit caveats.

#### Scenario: Index model is deterministic

r[octet.evidence_codebase_index.model.deterministic]

- GIVEN the same ordered set of supported evidence artifacts and the same normalization version
- WHEN the evidence index is built twice
- THEN the emitted index records and top-level index hash MUST be identical.
- AND local absolute paths, timestamps, usernames, and secret-like values MUST NOT affect the index hash.

### Requirement: Evidence artifact ingestion

r[octet.evidence_codebase_index.ingest] Octet MUST ingest existing supported evidence artifacts without changing their source formats.

#### Scenario: Supported artifacts are linked

r[octet.evidence_codebase_index.ingest.supported]

- GIVEN an artifact directory contains current Octet finding sidecars, provenance records, proof cards, replay capsules, transcript assertions, and fingerprint review manifests
- WHEN index ingestion runs
- THEN the index MUST include records for each supported artifact family.
- AND the records MUST link shared requirement IDs, lint IDs, policy IDs, source fingerprints, artifact hashes, and parent artifacts where the source artifact carries that data.

#### Scenario: Malformed artifacts fail closed or caveat

r[octet.evidence_codebase_index.ingest.malformed]

- GIVEN an artifact is missing required identity fields, has a stale BLAKE3 digest, is malformed, or belongs to an unsupported family
- WHEN index ingestion runs in strict mode
- THEN ingestion MUST fail closed with a deterministic issue code naming the artifact family and field.
- AND in advisory mode the index MAY keep a caveated partial record but MUST NOT invent missing identities.

### Requirement: Evidence index query surface

r[octet.evidence_codebase_index.query] Octet SHOULD expose deterministic query output for reviewers and scripts to find evidence by requirement ID, lint ID, policy ID, fingerprint, artifact hash, provenance source ID, or replay capsule identity.

#### Scenario: Query finds linked evidence

r[octet.evidence_codebase_index.query.lookup]

- GIVEN an index contains linked records for a requirement, lint finding, source fingerprint, provenance record, and replay capsule
- WHEN a reviewer queries any one of those stable identities
- THEN the query SHOULD return the linked records, artifact locators, evidence tiers, replay commands when available, parent links, caveats, and explicit non-claim text.

### Requirement: Static evidence browser

r[octet.evidence_codebase_index.html] Octet SHOULD render a static local HTML review surface from the same normalized index model as the query output.

#### Scenario: HTML review surface is navigable

r[octet.evidence_codebase_index.html.review_surface]

- GIVEN the evidence index contains findings, lint metadata, source fingerprints, configs, transcripts, proof cards, and provenance records
- WHEN the static browser is rendered
- THEN the generated pages SHOULD let a reviewer navigate from finding to lint reference, source fingerprint, config identity, transcript or proof receipt, and provenance parent chain.
- AND the generated pages SHOULD be deterministic for the same index input.

### Requirement: Evidence index boundaries

r[octet.evidence_codebase_index.boundaries] Evidence index output MUST state that it proves artifact identity and linkage only unless a linked formal-proof receipt provides a scoped stronger claim.

#### Scenario: Index avoids overclaims

r[octet.evidence_codebase_index.boundaries.no_overclaim]

- GIVEN generated JSON, human, or HTML index output describes evidence links
- WHEN claim-boundary validation scans the output
- THEN the output MUST NOT claim whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, or formal proof for non-proof evidence tiers.

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
