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

### Requirement: Typed multi-hop snapshot adapter

r[octet.evidence_codebase_index.multihop.snapshot] Octet SHOULD define an opt-in finite snapshot adapter over an already-built evidence codebase index that preserves the exact index hash, schema and normalization versions, source revisions, record occurrences, stable IDs, relation provenance, and an explicit snapshot completeness declaration.

#### Scenario: Snapshot preserves exact revision facts

r[octet.evidence_codebase_index.multihop.snapshot.preserved]

- GIVEN an evidence codebase index with an index hash, records, artifact locators, and BLAKE3 revisions
- WHEN a multi-hop snapshot is built
- THEN every occurrence MUST keep its record ID, stable ID, artifact family, locator, and source revision
- AND every relation MUST name the record field and value that produced it
- AND the report MUST name the snapshot hash and the index boundary verbatim.

#### Scenario: Snapshot forecloses unproven completeness

r[octet.evidence_codebase_index.multihop.snapshot.partial]

- GIVEN a snapshot without a completeness declaration, with a partial declaration, or with a missing index hash
- WHEN the snapshot is built
- THEN it MUST record a deterministic completeness gap
- AND a query over it MUST report an incomplete snapshot rather than a complete result.

### Requirement: Relation semantics stay distinct

r[octet.evidence_codebase_index.multihop.relations] Octet MUST keep explicit parent links, provenance references, asserted dependency/support, shared labels, and content identity distinct in multi-hop output.

#### Scenario: Shared labels remain relatedness

r[octet.evidence_codebase_index.multihop.relations.shared_label]

- GIVEN two records share only a requirement, policy, or lint ID
- WHEN a causal impact query admits parent links only
- THEN neither record MUST be reached from the other
- AND when a relatedness query admits the shared-label kind, the reached reason MUST carry shared-label semantics with no dependency or proof-support claim.

#### Scenario: Content identity is not proof

r[octet.evidence_codebase_index.multihop.relations.content_identity]

- GIVEN two records share an artifact hash or fingerprint
- WHEN the content-identity relation is admitted
- THEN reachability MAY report the identity relation
- AND the reason MUST NOT claim dependency or proof support.

#### Scenario: Asserted support is unavailable

r[octet.evidence_codebase_index.multihop.relations.asserted_support]

- GIVEN the evidence index model carries no asserted dependency/support field
- WHEN a query admits asserted support
- THEN the report MUST mark that relation unavailable with an explicit reason
- AND no parent-link, label, provenance, or identity edge MUST be relabeled as asserted support.

### Requirement: Bounded reachability returns typed reasons and a selected witness

r[octet.evidence_codebase_index.multihop.reachability] Octet SHOULD evaluate bounded recursive reachability over admitted relation kinds and return reached record occurrences, typed reasons, and one deterministic selected witness with explicitly classified closures.

#### Scenario: Multi-hop chain extends beyond one expansion

r[octet.evidence_codebase_index.multihop.reachability.chain]

- GIVEN an index whose records form a parent-link chain longer than one expansion
- WHEN the typed reference evaluates the chain root
- THEN it MUST return every chain record with its depth and typed edge reason
- AND the selected witness MUST contain one supporting edge per reached non-root record
- AND the existing one-expansion query MUST remain unchanged.

#### Scenario: Diamonds, cycles, and self loops are explicit

r[octet.evidence_codebase_index.multihop.reachability.closures]

- GIVEN a diamond, a cycle, and a self edge
- WHEN reachability evaluates from the covered origins
- THEN a diamond MUST select one predecessor and record one diamond closure
- AND a cycle MUST record one cycle closure without re-enqueueing the ancestor
- AND a self edge MUST record one self-loop closure
- AND the origin MUST remain reachable at depth zero.

#### Scenario: Evaluators agree

r[octet.evidence_codebase_index.multihop.reachability.agreement]

- GIVEN the same snapshot, admitted relation kinds, and limits
- WHEN the typed reference and the fixed-rule Datalog-equivalent evaluator run
- THEN their reached occurrences, selected witness edges, closures, and status MUST agree.

### Requirement: Bounds and truncation are distinguishable

r[octet.evidence_codebase_index.multihop.limits] Octet MUST bound depth, facts, tuples, work, estimated memory, and explanation output, and MUST distinguish analysis truncation from explanation truncation and from an incomplete snapshot.

#### Scenario: Analysis limits stop traversal

r[octet.evidence_codebase_index.multihop.limits.analysis]

- GIVEN a reachable chain and a depth, fact, tuple, work, or memory bound below the chain requirement
- WHEN reachability runs
- THEN the report MUST stop under that named limit with an analysis-stopping truncation
- AND the status MUST report truncation
- AND reached records MUST be limited to what was established before the stop.

#### Scenario: Output truncation preserves analysis

r[octet.evidence_codebase_index.multihop.limits.output]

- GIVEN a complete bounded analysis whose explanation exceeds the output bound
- WHEN reachability runs
- THEN the reached record set MUST remain complete
- AND the witness MUST be marked truncated
- AND the status MUST remain complete when no analysis limit and no snapshot gap applies.

### Requirement: Runtime freshness and endpoint semantics are explicit

r[octet.evidence_codebase_index.multihop.freshness] Octet MUST evaluate each snapshot with fresh state and MUST reject a runtime bound to a different snapshot instead of returning stale conclusions.

#### Scenario: Replaced revision is not reused

r[octet.evidence_codebase_index.multihop.freshness.stale_runtime]

- GIVEN a runtime evaluated against one snapshot revision
- WHEN the same runtime is asked to evaluate a snapshot with a replaced source revision, removed link, or different index hash
- THEN it MUST return a stale-snapshot error
- AND a fresh runtime for the new snapshot MUST reflect only the new snapshot.

#### Scenario: Ambiguous and unknown endpoints stay unresolved

r[octet.evidence_codebase_index.multihop.freshness.endpoints]

- GIVEN duplicate stable IDs, a parent link to a missing artifact, or a parent link that resolves to several occurrences
- WHEN a snapshot is built
- THEN duplicate occurrences, unknown endpoints, and ambiguous endpoints MUST each be reported explicitly
- AND no ambiguous endpoint MUST be guessed into a single edge.

#### Scenario: Conflicting revisions are explicit

r[octet.evidence_codebase_index.multihop.freshness.conflict]

- GIVEN two records name the same artifact locator with different BLAKE3 revisions
- WHEN a snapshot is built
- THEN the conflict MUST be reported with both revisions
- AND a query over the snapshot MUST report an incomplete snapshot.

### Requirement: Existing query remains the compatibility baseline

r[octet.evidence_codebase_index.multihop.compatibility] The multi-hop experiment MUST NOT change `query_evidence_index`, its report shape, its return values, source discovery, process execution, or publication.

#### Scenario: Baseline query is unchanged

r[octet.evidence_codebase_index.multihop.compatibility.baseline]

- GIVEN a parent-link chain longer than one expansion
- WHEN the existing query runs from the chain root
- THEN it MUST return only the root and its direct linked record
- AND the multi-hop experiment MUST be reachable only through its opt-in API
- AND no report MUST claim whole-program correctness, semantic equivalence, complete dependency closure, release eligibility, or formal proof.

### Requirement: Dependency decision is recorded

r[octet.evidence_codebase_index.multihop.decision] Octet MUST record a keep, revise, or stop decision for an external Datalog dependency and MUST keep the typed traversal when composed rules do not justify the dependency.

#### Scenario: Decision cites measured value

r[octet.evidence_codebase_index.multihop.decision.stop]

- GIVEN the fixed-rule program and the typed traversal examine the same edges, derive the same occurrences, and return the same witness on the selected fixtures
- WHEN the dependency decision is reviewed
- THEN the recorded decision MUST be stop
- AND no Datalog dependency MUST be added
- AND the typed traversal MUST remain the implementation.

#### Scenario: Decision is reopened only on new evidence

r[octet.evidence_codebase_index.multihop.decision.reopen]

- GIVEN a future request proposes a Datalog dependency
- WHEN review runs
- THEN it MUST name composed rules over several relation families that the typed traversal cannot express simply
- AND it MUST name asserted-support facts that the index actually carries
- AND agreement, limit, and freshness tests MUST cover the new rules before the decision changes.
