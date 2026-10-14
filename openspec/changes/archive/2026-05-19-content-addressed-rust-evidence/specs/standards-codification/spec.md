# Content Addressed Rust Evidence Delta

## ADDED Requirements

### Requirement: Octet-owned function object v1

r[content_addressed_rust_evidence.function_object_v1] Octet MUST emit deterministic function-object v1 records for selected Rust functions as source-token review/provenance identities with explicit schema, normalization, toolchain, body hash, signature hash, object hash, source span, semantic id, and caveats.

#### Scenario: Emit source-token function object

- GIVEN a selected Rust function accepted by Octet's fingerprint scanner
- WHEN the user requests a function-object v1 record
- THEN Octet emits canonical JSON containing the schema id, semantic id, source span, body hash, signature hash, normalization version, toolchain identity, object hash, and caveats declaring source-token identity.

#### Scenario: Reject correctness overclaim

- GIVEN a function-object v1 record whose hashes match current source
- WHEN Octet renders JSON, human output, or review text for that object
- THEN the output describes the object as review/provenance evidence and not as a proof of behavioral correctness or semantic equivalence.

#### Scenario: Real-code corpus emits stable source objects

- GIVEN a bounded corpus of Octet's real Rust implementation modules
- WHEN Octet emits function-object v1 records across that corpus twice
- THEN the emitted object set is deterministic, covers many real functions, preserves per-file source paths, reports caveat distribution, and keeps correctness claims false.

### Requirement: Canonical object hashing

r[content_addressed_rust_evidence.canonical_hash] Octet MUST compute object hashes with BLAKE3 over deterministic canonical object content that excludes self-referential hash fields and ambient entropy.

#### Scenario: Same logical object hashes the same across roots

- GIVEN the same logical selected function emitted from two different repository root paths
- WHEN Octet computes the function-object v1 hash
- THEN display-only path differences do not change the hash input and the object hashes match.

#### Scenario: Entropy-bearing fields are excluded or rejected

- GIVEN object fields derived from timestamps, temporary absolute paths, environment variables, or process-local state
- WHEN Octet prepares canonical hash input
- THEN those fields are either excluded as display metadata or rejected as non-canonical hash material.

### Requirement: Object marker compatibility

r[content_addressed_rust_evidence.object_markers] Octet MUST support inline object markers that bind selected Rust functions to function-object hashes while preserving existing fingerprint marker workflows.

#### Scenario: Object marker check detects drift

- GIVEN a function with an inline object marker referencing a prior object hash
- WHEN the function body, signature, or caveat-relevant context changes
- THEN Octet check reports drift and requires an explicit update before the object is considered reviewed.

#### Scenario: Existing fingerprint workflows remain usable

- GIVEN a repository that only uses existing Octet function fingerprint markers
- WHEN object support is added
- THEN existing fingerprint check, write, update, and diff workflows continue to work without requiring object markers.

### Requirement: Name pointer registry

r[content_addressed_rust_evidence.name_pointers] Octet MUST support a deterministic name-pointer manifest that maps mutable Rust semantic names to immutable object hashes and records pointer movement metadata.

#### Scenario: Pointer move records old and new identity

- GIVEN a name pointer for `crate::module::function` that references an old object hash
- WHEN the user moves that pointer to a new object hash
- THEN Octet records the old hash, new hash, semantic name, diff summary, and rollback handle in deterministic output.

#### Scenario: Pointer references must resolve

- GIVEN a name-pointer manifest and an object bundle
- WHEN Octet validates the manifest
- THEN every pointer hash resolves to a present object record or validation fails closed.

### Requirement: Structured dependency and effect caveats

r[content_addressed_rust_evidence.caveats] Octet MUST emit machine-readable dependency, macro, generated-code, and effect caveats using closed status values that distinguish source-token scanning from future HIR/MIR evidence.

#### Scenario: Source-scanned dependencies are not reported as resolved

- GIVEN a function-object v1 record produced by the source scanner
- WHEN Octet emits dependency metadata
- THEN the dependency status uses a source-scanner status and does not claim rustc/HIR-resolved dependency identity.

#### Scenario: Unknown effects block pure-cache eligibility

- GIVEN a function object with unknown, macro-hidden, generated, or unresolved effect sources
- WHEN Octet emits a pure-cache eligibility summary
- THEN the object is reported as ineligible unless stronger reviewed evidence is present.

### Requirement: Object diff

r[content_addressed_rust_evidence.object_diff] Octet MUST provide object-aware diffs that report body, signature, dependency caveat, effect caveat, macro caveat, and name-pointer changes separately.

#### Scenario: Diff separates body and caveat changes

- GIVEN old and new function-object records for the same semantic name
- WHEN Octet compares them
- THEN the diff reports whether body hash, signature hash, dependency caveats, effect caveats, macro caveats, or pointer targets changed as separate fields.

### Requirement: Export bundle

r[content_addressed_rust_evidence.export_bundle] Octet MUST export deterministic evidence bundles for downstream tools containing object records, name pointers, review manifests, caveat summaries, and replay metadata.

#### Scenario: Bundle membership is deterministic

- GIVEN a set of selected function objects and name pointers
- WHEN Octet exports a bundle
- THEN bundle membership and manifest ordering are deterministic and the bundle hash is stable for the same logical content.

#### Scenario: Bundle records replay command

- GIVEN an exported bundle
- WHEN a downstream consumer reads its receipt metadata
- THEN the metadata records the Octet command, relevant config/normalization versions, and input selection needed to reproduce the bundle.

#### Scenario: Real-code corpus bundle is deterministic

- GIVEN function objects emitted from a bounded corpus of Octet's real Rust implementation modules
- WHEN Octet exports the corpus bundle repeatedly
- THEN the bundle bytes, object count, bundle hash, and replay command are stable for the same corpus selection.

### Requirement: Real-code harness hardening

r[content_addressed_rust_evidence.real_code_harness] Octet MUST provide a repeatable real-code harness for function-object emission, caveat summaries, negative case classification, and bundle determinism before adding compiler IR-derived evidence.

#### Scenario: Repeatable real-code harness emits classified summary

- GIVEN a named corpus manifest of Octet implementation source files
- WHEN the harness emits function-object v1 records repeatedly
- THEN it compares stable object output, reports object count, source-path coverage, macro/effect caveat distribution, pure-cache blocking, and no-correctness-overclaim status.

#### Scenario: Negative corpus cases are classified

- GIVEN fixture mutations for body changes, comment/formatting-only changes, macro-heavy functions, missing pointers, and unresolved bundle pointers
- WHEN the harness runs those cases
- THEN it proves expected hash stability or drift and fails closed with classified caveat or missing-object outcomes.

#### Scenario: Harness bundle export is deterministic

- GIVEN objects emitted from the named real-code corpus
- WHEN the harness exports bundles repeatedly
- THEN bundle bytes, object count, pointer count, bundle hash, and replay command are stable, and unresolved pointer manifests fail closed.

#### Scenario: Strong local determinism receipt

- GIVEN a scrubbed local environment, multiple fresh target directories, a copied corpus root, and permuted corpus input order
- WHEN the harness emits objects and bundles ten times
- THEN the receipt records binary, toolchain, system, corpus file hashes, stable emit byte hash, stable bundle byte hash, cross-root object-set equivalence, and input-order object-set equivalence.

### Requirement: Compiler IR evidence roadmap

r[content_addressed_rust_evidence.compiler_ir] Octet MUST treat future HIR, THIR, and MIR evidence as supplemental compiler-version-coupled evidence layers that improve dependency/effect summaries without replacing function-object v1 review identity.

#### Scenario: HIR evidence is toolchain scoped

- GIVEN a future object containing HIR-derived dependency evidence
- WHEN Octet emits that evidence
- THEN the record includes rustc/toolchain identity, extractor version, resolved dependency summary, and unresolved or macro-origin caveats.

#### Scenario: MIR evidence does not claim cross-version stability

- GIVEN a future object containing MIR-derived call, panic, drop, async, or unsafe summaries
- WHEN Octet emits that evidence
- THEN the record identifies the exact rustc/toolchain and avoids claiming stability across compiler versions.

### Requirement: Downstream evidence boundaries

r[content_addressed_rust_evidence.downstream_boundaries] Octet MUST define downstream evidence boundaries for Valence, Cairn, Trellis, Crunch/Mantle, and Aspen that preserve Octet as the Rust extraction/object source while avoiding claims owned by those systems.

#### Scenario: Cairn consumes Octet receipts without owning extraction

- GIVEN an Octet export bundle receipt
- WHEN Cairn consumes it
- THEN Cairn validates lifecycle/replay policy around the receipt while Octet remains the authority for Rust extraction and object emission.

#### Scenario: Crunch and Aspen pilots remain bounded

- GIVEN Crunch/Mantle or Aspen consumes Octet evidence objects
- WHEN Octet describes the interop contract
- THEN the contract identifies checker/config/source/executor hashes as evidence identity and does not claim build correctness or distributed execution correctness from Octet hashes alone.
