## ADDED Requirements

### Requirement: Reviewed source-token-v2 Function Object construction

The system MUST construct new source-backed Function Object records only with corrected canonical tokens and an independently reviewed Valence normalization identity `octet-source-token-v2`; it MUST NOT relabel old v1 bytes, present corrected bytes as v1, or emit an object when the exact pinned Valence API cannot admit v2. The `octet.function-object.v1` record schema MAY remain unchanged if the reviewed external contract supports it, but the normalization identity MUST participate in the canonical object hash, and evidence-only caveats MUST remain explicit.

#### Scenario: Reviewed pin unlocks source-backed object emission

- GIVEN an exact Cargo/Nix/Valence cutover-manifest pin to a reviewed core that admits `octet-source-token-v2`
- WHEN `cargo octet object emit --function fetch --output-format json src/lib.rs` selects a function
- THEN exactly one object contains `normalization.algorithm="octet-source-token-v2"`, a BLAKE3 object hash, and `claims_correctness=false`; marker, corpus, and bounded supplement calls use the same constructor.

#### Scenario: Quoted URL and literal changes alter v2 identity

- GIVEN selected Rust source with a quoted `"https://example.invalid/api"` before executable code in `fetch`
- WHEN the return expression after the URL changes, or literal contents change without changing surrounding tokens
- THEN `object emit` still discovers `fetch` and its object hash changes for either edit.

#### Scenario: Trivia leaves v2 identity stable

- GIVEN a selected function whose Rust token boundaries, punctuation jointness, and literal contents are unchanged
- WHEN ordinary line or nested block comments and whitespace change without altering the compiler's token stream
- THEN the canonical object hash stays stable, while unresolved dependency/macro/effect caveats continue to block unsupported purity or correctness claims.

#### Scenario: Macro lexical token boundaries change v2 identity

- GIVEN pinned-rustc-valid function bodies with macro arguments differing in joint versus separate punctuation, numeric literal versus separate tokens, raw identifier or lifetime versus separate tokens, attached versus separate raw-literal suffix, or doc-comment attribute versus ordinary comment
- WHEN `cargo octet object emit` computes each function's source-token-v2 identity
- THEN distinct compiler-visible macro token streams produce distinct normalized-body and object hashes, while edits that leave the token stream unchanged (including ordinary comments and whitespace next to groups) preserve identity; a macro caveat MUST NOT excuse a collision or imply correctness.

#### Scenario: Marker-looking text inside non-code cannot confer review

- GIVEN a function with an apparently matching `// octet-object:` or fingerprint comment/attribute embedded in a Rust block comment or literal, and a separate function with a real reviewed marker above outer attributes
- WHEN Object check or function-address deny evaluates those sources
- THEN forged text yields missing marker/review and a nonzero check rather than match or stale; the real marker still attaches, and explicit update inserts an active marker outside any inline comment closer without inventing review metadata.

### Requirement: Versioned object corpus compatibility fixtures

The system MUST retain the checked-in v1 receipt, its source, and negative compatibility fixtures as immutable historical artifacts, and MUST provide a separately named v2 source/receipt pair at `cargo-octet/tests/fixtures/object-corpus/source-token-v2/{src/lib.rs,positive-v2-receipt.json}`. The owner-reviewed v2 source manifest MUST additionally bind this fixed fixture's source path/bytes and receipt digest. A receipt with a valid v1 schema MAY be parsed, but replay against newly emitted v2 objects MUST compare identities instead of silently rebinding the v1 object-set hash. `status="match"` MUST be reserved for a receipt actually emitted for the same versioned source identity. Neither matching nor readable artifacts grant review approval. The checked-in v1 receipt does not contain standalone serialized Function Objects; artifact-only object operations MAY be demonstrated with explicitly test-owned serialized records from the real typed v1 Valence constructor, never represented as archived v1 output or reconstructed from current v2 emission.

#### Scenario: Historical v1 receipt and v1-shape artifacts remain readable

- GIVEN the unchanged `compatibility/positive-v1-receipt.json`, a Valence core golden known-answer test captured from the immutable v1 parent, and two explicitly test-owned, typed `octet.function-object.v1` records serialized by the public Valence v1 constructor (not previously saved standalone v1 objects)
- WHEN artifact-only JSON reading examines the saved receipt, the pinned-v1 golden test runs, and object diff, pointer validation, and bundle export read the test-owned record without source replay
- THEN the historical receipt's fields and object-set hash remain intact, the independently pinned golden hash authenticates the first typed v1 record, and artifact-only operations preserve its real serialized fields and hash while diff detects the second record's body change; neither object hash is claimed to match the historical receipt. No source marker, pointer, baseline, or receipt is automatically rewritten. The separate historical source-replay scenario proves that the receipt remains shape-valid while its object-set identity mismatches v2.

#### Scenario: Historical v1 source replay is stale under v2

- GIVEN the unchanged `compatibility/positive-v1-receipt.json` and its checked-in `src/lib.rs`
- WHEN `cargo octet object corpus check --receipt positive-v1-receipt.json --output-format json src/lib.rs` constructs current v2 objects
- THEN it exits nonzero with `status="mismatch"`, a comparison naming `object_set_hash`, and no false `status="match"`; v1 JSON remains valid historical input, not `invalid` solely because its schema is v1.

#### Scenario: Positive v2 fixture replays cleanly

- GIVEN the separately reviewed `source-token-v2/positive-v2-receipt.json`, its quoted-URL source, and their approved source-manifest bindings
- WHEN `object corpus check` replays it against that same source
- THEN it returns `status="match"`, all receipt-critical comparisons match, and both boundary claims remain false.

#### Scenario: V2 source drift changes object-set identity

- GIVEN a saved v2 corpus receipt and its original source
- WHEN the URL literal or executable code after the quoted URL changes before replay
- THEN check exits nonzero with `status="mismatch"` on `object_set_hash`, without claiming semantic equivalence.

#### Scenario: V2 boundary tampering fails closed

- GIVEN a valid v2 receipt whose `boundary.claims_correctness` or `boundary.claims_semantic_equivalence` was changed to true
- WHEN the receipt is checked against unchanged source
- THEN check exits nonzero with `status="mismatch"` on `boundary` and cannot report a passing gate.

#### Scenario: Negative fixtures reject unsupported compatibility shapes

- GIVEN the retained v1 negative fixtures for missing fields, future version, duplicate source path, empty selection, malformed JSON, or unsupported non-Rust source
- WHEN `object corpus check` validates each fixture
- THEN it returns `status="invalid"` with the matching typed error and no comparisons, before attempting source-backed v2 construction.

### Requirement: Versioned Internal Object Corpus Gate

The system MUST compare the selected internal Rust source set to a separately stored, owner-reviewed v2 reference receipt at `cargo-octet/tests/fixtures/object-corpus/source-token-v2/internal-corpus-receipt.json`. An independent `source-token-v2/reviewed-source-manifest.json` MUST bind the exact sorted repo-relative source paths and BLAKE3 of every selected source file, `octet-source-token-v2`, the reviewed Valence revision, Octet version, reference receipt path and BLAKE3, and an explicit owner review reference. Selection, source bytes, pin, receipt, or approval-reference drift MUST fail the named `object-corpus-gate` before it reports PASS. Fresh regeneration and self-replay MAY additionally check determinism but MUST NOT replace comparison with the stored reference. The historical `internal-corpus-receipt.json` v1 fixture MUST remain unchanged.

#### Scenario: Bounded internal v2 corpus matches reviewed source-bound reference

- GIVEN an independently owner-reviewed internal v2 receipt and source manifest for a nonempty selected Octet corpus
- WHEN `object-corpus-gate` measures the actual source paths/bytes, emits current v2 objects, and replays the saved receipt
- THEN all bindings match and both boundary claims remain false with unresolved blockers visible; a missing reference, stale source byte, changed selected path, missing approval reference, or changed receipt yields a nonzero result even when two fresh receipts agree with each other.

### Requirement: Versioned expanded internal object corpus taxonomy

The system MUST retain the historical expanded v1 receipt and taxonomy unchanged and compare the selected current `cargo-octet` and `octet-standards` source sets against separately stored, owner-reviewed `source-token-v2/expanded-internal-corpus-receipt.json` and `source-token-v2/expanded-internal-corpus-caveat-taxonomy.json`. The same approved source manifest MUST bind sorted source paths and their exact BLAKE3 bytes plus each v2 reference path/digest. The named caveat-taxonomy gate MUST fail on missing/unapproved references or selection/source/reference drift, not substitute a report freshly generated from that same source as the expected result.

#### Scenario: Expanded v2 corpus and taxonomy match reviewed source-bound references

- GIVEN the separately approved expanded v2 receipt and taxonomy with a source manifest covering both `cargo-octet` and `octet-standards` buckets
- WHEN the named caveat-taxonomy gate emits a current receipt and per-source taxonomy
- THEN actual source paths/bytes and generated receipt/taxonomy match both independent references, their boundary claims are false, unresolved blockers remain visible, and any stale/missing reference or source fails even when freshly generated copies agree; expanded v1 JSON is neither rewritten nor used as expected v2 output.

### Requirement: Versioned fresh-build object corpus determinism rail

The system MUST compare v2 corpus receipts generated by two independently built Octet binaries against each other AND against the independently owner-reviewed internal/expanded v2 reference receipts bound to actual source bytes by the approved manifest. The fixed quoted-URL v2 compatibility fixture MAY be an additional control, not the sole reviewed source. The rail MUST fail closed if any selected source, manifest approval, reference, or boundary is stale or missing; `docs/contributor-guide.md` MUST name the existing fresh-build rail and require owner review before refreshing v2 references. It MUST NOT claim parity with or modify historical expanded v1 JSON.

#### Scenario: Fresh-build binaries match reviewed internal and expanded v2 references

- GIVEN two independent binaries, approved source manifest, and stored internal/expanded v2 reference receipts
- WHEN `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L` runs
- THEN both binaries' corresponding receipts are byte-identical to each other AND their reviewed references for unchanged source, with both correctness/semantic-equivalence claims false; source or reference drift fails despite inter-binary agreement, and no cross-machine/toolchain claim or v1 fixture rewrite occurs.

### Requirement: Versioned object corpus fixture freshness rail

The system MUST reuse the existing named `object-corpus-fixture-freshness` rail, not introduce a substitute gate. That rail MUST check the independently approved internal/expanded v2 source manifest and receipts, expanded caveat taxonomy, and fixed `source-token-v2/{src/lib.rs,positive-v2-receipt.json}` pair. It MUST reject absent/unapproved references, changed source bytes, stale output, algorithm/version drift, or zero selected behavioral tests instead of self-generating expected values. `docs/contributor-guide.md` and `openspec/templates/tasks.md` MUST describe intentional owner review of new v2 reference refreshes and preservation of historical v1 fixtures.

#### Scenario: Named fixture freshness rail compares reviewed v2 references

- GIVEN approved source manifest, separately stored internal/expanded v2 references and fixed quoted-URL v2 fixture
- WHEN `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L` runs
- THEN a nonignored source-backed test checks actual source hashes/selection against the approved manifest and fresh v2 output against each saved reference, including false boundary claims; a deliberately stale source/reference or missing owner review fails rather than yielding PASS.

### Requirement: Selected v2 Object evidence gates

The system MUST select and run nonignored, source-backed tests for every named object/corpus/supplement Nix child rail, including literal/quoted-URL behavior and positive/negative replay; the aggregate MUST not print any object-evidence PASS if a selected test, fresh-build comparison, or exact Valence dependency provenance check fails. Historical `docs/current-head-valence-evidence.md` MAY continue to describe its attested `c9cdf676` payload but MUST NOT count as proof that the new ObjectV1 v2 path currently passes.

#### Scenario: Selected child rails exercise real source-bound v2 behavior

- GIVEN the object-corpus, compatibility, caveat-taxonomy, freshness, supplement-compatibility, THIR-inventory, MIR-fixture, and MIR-inventory named checks
- WHEN the Nix rails run
- THEN their filters select nonignored source-backed emission/replay and meaningful stale-source, stale-reference, or receipt-tamper negatives, and internal/expanded checks compare independently approved source-bound references; empty selection, refusal-only tests, self-produced expected receipts, or artifact-only checks cannot make a child PASS.

#### Scenario: Aggregate PASS requires all selected source-bound v2 rails

- GIVEN the named child rails, fresh-build check, exact standalone Valence cutover check, and historical evidence-index/release checks
- WHEN `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` runs
- THEN it emits named PASS lines including the existing fixture freshness rail only if all source-bound references and child checks succeed; a missing/unapproved internal or expanded v2 reference blocks the aggregate without any PASS, preserves existing non-proof scope, and does not treat historical `c9cdf676` evidence as new v2 verification.

## REMOVED Requirements

### Requirement: Object corpus compatibility fixtures

This requirement is replaced by `Versioned object corpus compatibility fixtures` only after explicit owner approval: its `Positive v1 fixture replays cleanly` scenario incorrectly requires a historical v1 object-set hash to match default v2 source construction. The replacement retains v1 artifact readability and invalid-shape rejection, requires historical v1 source replay to report a mismatch, and adds independently reviewed positive v2 replay.

### Requirement: Internal Object Corpus Gate

This requirement is replaced by `Versioned Internal Object Corpus Gate`: current source-backed v2 tests must match a separate owner-reviewed source-byte-bound reference, not self-produced receipts or the historical `internal-corpus-receipt.json` v1 hash.

### Requirement: Expanded internal object corpus taxonomy

This requirement is replaced by `Versioned expanded internal object corpus taxonomy`: historical v1 receipt/taxonomy stay immutable while current selected Octet source buckets are checked against independent owner-reviewed v2 receipt/taxonomy references.

### Requirement: Fresh-build object corpus determinism rail

This requirement is replaced by `Versioned fresh-build object corpus determinism rail`: independent v2 binaries must agree with each other AND owner-reviewed internal/expanded v2 references, never claim parity with an expanded v1 receipt.

### Requirement: Object corpus fixture freshness rail

This requirement is replaced by `Versioned object corpus fixture freshness rail`: the existing named rail checks reviewed internal/expanded v2 references and the fixed quoted-URL fixture; old v1 JSON remains historical.
