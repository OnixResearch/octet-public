# Evidence codebase index

`cargo octet evidence-index codebase` builds a local, deterministic Octet-owned review index over existing Octet evidence artifacts. It is a map for reviewers and scripts: finding, lint, policy, requirement, fingerprint, provenance, proof-card, replay-capsule, transcript, and evidence-log identities link back to concrete artifact files and replay commands.

The pure indexing core is currently hosted in `valence-core::evidence_codebase_index` as an in-repository deterministic helper. Octet retains CLI parsing, filesystem traversal, artifact discovery, output writing, Nix gate ownership, release bundle ownership, and non-claim boundary enforcement; hosting the pure core in `valence-core` does not imply package publication, standalone Valence repository cutover, or transfer of Octet evidence release responsibilities.

## Build

```sh
cargo octet evidence-index codebase build \
  --artifact-dir artifacts/octet \
  --output artifacts/octet/evidence-codebase-index.json
```

Use repeated `--artifact PATH` for explicit files and repeated `--artifact-dir DIR` for directories. Directory scans include supported `.json`, `.jsonl`, and `.md` evidence files and skip `.git`, `.jj`, and `target` directories.

Strict mode is the default. Missing required identity fields, malformed JSON, stale supplied artifact hashes, and unsupported artifact families are reported as deterministic `error` issues and the command exits non-zero. `--advisory` records those conditions as caveats for exploratory review without inventing missing identities.

## Query

```sh
cargo octet evidence-index codebase query \
  --index artifacts/octet/evidence-codebase-index.json \
  --identity 'r[octet.evidence_codebase_index.query.lookup]'
```

Queries match stable record IDs, artifact locators, BLAKE3 artifact hashes, requirement IDs, lint IDs, policy IDs, fingerprints, provenance source IDs, and replay capsule identities. Query output includes directly matching records and deterministic linked records, including parent artifacts and shared requirement/lint/policy/fingerprint links.

## Static HTML

```sh
cargo octet evidence-index codebase html \
  --index artifacts/octet/evidence-codebase-index.json \
  --output artifacts/octet/evidence-codebase-index.html
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
