# Evidence codebase index

`cargo octet evidence-index codebase` builds a local, deterministic Octet-owned review index over existing Octet evidence artifacts. It is a map for reviewers and scripts: finding, lint, policy, requirement, fingerprint, provenance, proof-card, replay-capsule, transcript, and evidence-log identities link back to concrete artifact files and replay commands.

The pure indexing core is consumed from the exact standalone `valence-core::evidence_codebase_index` dependency through Octet's compatibility shim. Octet retains CLI parsing, filesystem traversal, artifact discovery, output writing, Nix gate ownership, release bundle ownership, and non-claim boundary enforcement; standalone Valence ownership of the pure core does not transfer Octet's product or release responsibilities.

## Build

```sh
cargo octet check --output-format json --artifact-dir artifacts/octet
cargo octet evidence-index codebase build \
  --artifact artifacts/octet/results.jsonl \
  --artifact artifacts/octet/provenance.jsonl \
  --output target/octet-evidence-index/evidence-codebase-index.json
```

Use repeated `--artifact PATH` for explicit files and repeated `--artifact-dir DIR` for directories. Directory scans include supported `.json`, `.jsonl`, and `.md` evidence files and skip `.git`, `.jj`, and `target` directories.

Write the index and the HTML page outside every directory passed to `--artifact-dir` and outside Octet artifact bundles such as `artifacts/octet`: `cargo octet artifact verify --artifact-dir artifacts/octet` reports any file it did not write as an unexpected artifact. `target/` is a good home because directory scans skip it. A build never reads its own `--output` file back from a scanned directory.

A `cargo octet check` bundle also holds runner records (`status.json`, `command.txt`, `summary.txt`). No supported artifact family covers `status.json`, so `--artifact-dir artifacts/octet` over a check bundle reports it as `unsupported_family` in strict mode; name the bundle's `results.jsonl` (written by `--output-format json`) and `provenance.jsonl` with `--artifact` instead. A check with no findings writes an empty `results.jsonl`, which strict mode also reports as `unsupported_family`; index only `provenance.jsonl` in that case. The provenance records still name the runner files, which appear as `provenance.referenced_artifact_missing` caveats.

Strict mode is the default. Missing required identity fields, malformed JSON, stale supplied artifact hashes, and unsupported artifact families are reported as deterministic `error` issues and the command exits non-zero. `--advisory` records those conditions as caveats for exploratory review without inventing missing identities.

Each action accepts only its own flags; for example, `query` rejects `--output` and `--advisory` instead of ignoring them.

## Query

```sh
cargo octet evidence-index codebase query \
  --index target/octet-evidence-index/evidence-codebase-index.json \
  --identity octet.runner.artifact-bundle
```

Queries match stable record IDs, artifact locators, BLAKE3 artifact hashes, requirement IDs such as `r[octet.evidence_codebase_index.query.lookup]`, lint IDs, policy IDs, fingerprints, provenance source IDs such as the check bundle's `octet.runner.artifact-bundle`, and replay capsule identities. Query output includes directly matching records and deterministic linked records, including parent artifacts and shared requirement/lint/policy/fingerprint links. Query prints to standard output and exits non-zero when no record matches.

## Typed multi-hop experiment (opt-in)

`cargo-octet` exposes an experimental library module, `evidence_multihop`, over the same `EvidenceCodebaseIndex` model. It is not wired to `cargo octet check`, it adds no CLI action, and it does not change the query above.

The experiment builds one finite `EvidenceSnapshot` from an already-built index. The snapshot keeps the exact index hash, schema and normalization versions, source revisions, record occurrences, stable IDs, and the field/value provenance of every relation. It records an explicit completeness declaration; an undeclared or partial snapshot reports an incomplete result. Unknown or ambiguous endpoints, duplicate stable IDs, missing revisions, conflicting revisions for one locator, and index issues are reported rather than guessed away.

Relations stay separate:

- `parent_links` is an explicit parent link.
- `provenance_source_ids` is an explicit provenance reference.
- `asserted_support` is an asserted dependency/support relation. The evidence index has no such field, so the experiment reports it unavailable with a fixed reason and never relabels another relation as support.
- `requirement_ids`, `lint_ids`, and `policy_ids` are shared labels. They are relatedness only and never causal dependency.
- `artifact_hashes` and `fingerprints` are content identity. They are not proof support.

`evaluate_typed_reference` is the correctness baseline: a deterministic typed-edge breadth-first traversal. `evaluate_fixed_rule_datalog` is a bounded fixed-rule Datalog-equivalent evaluator with a fresh runtime per snapshot. `BoundedDatalogRuntime` rejects a different snapshot instead of returning stale conclusions. Both evaluators return reached record occurrences with typed reasons and one selected witness: one predecessor per reached occurrence plus back edges classified as cycle, self loop, or diamond closure. Paths are not enumerated.

`QueryLimits` bounds depth, facts, tuples, work, estimated memory, and explanation output. Depth, facts, tuples, work, and memory stop traversal and produce a truncated status. The output bound only shortens the witness; a complete analysis stays complete and `output_truncated` says the explanation is a bounded selection. An incomplete snapshot is a separate axis. The derived status is one of `complete`, `truncated`, `incomplete_snapshot`, or `truncated_and_incomplete_snapshot`.

The experiment stops the external Datalog dependency path. On the covered fixtures the fixed-rule program and the typed traversal examine the same edges, derive the same occurrences, and return the same witness, while the Datalog path adds rule instrumentation and a second traversal to keep in agreement. One rule program over one typed graph does not justify Crepe or another dependency, and the base index carries no asserted-support facts for composed rules to exploit. The typed traversal remains the implementation.

## Static HTML

```sh
cargo octet evidence-index codebase html \
  --index target/octet-evidence-index/evidence-codebase-index.json \
  --output target/octet-evidence-index/evidence-codebase-index.html
```

The HTML page is rendered from the same normalized index model as JSON query output. It is intended for local static browsing; no hosted service is required.

## Supported artifact families

The first implementation ingests current Octet surfaces without replacing their source formats:

- `cargo octet check` finding JSON/JSONL records.
- Octet provenance JSONL records.
- `cargo octet prove` proof-card decks.
- Function fingerprint review manifests.
- Replay capsules from `cargo octet roadmap capsule`.
- Transcript assertion reports and markdown transcripts.
- Evidence log manifests.
- Evidence rail receipts and their findings.

## Boundary

Every generated index and HTML page states the same non-claim boundary: the Octet evidence codebase index proves artifact identity and linkage only. It does not claim whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, standalone Valence release ownership, or formal proof unless a linked formal-proof receipt scopes that stronger claim.
