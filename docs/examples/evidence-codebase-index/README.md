# Evidence codebase index example

Build a deterministic local Octet-owned index over existing artifacts:

```sh
cargo octet evidence-index codebase build --artifact-dir artifacts/octet --output artifacts/octet/evidence-codebase-index.json
```

Query by a stable identity:

```sh
cargo octet evidence-index codebase query --index artifacts/octet/evidence-codebase-index.json --identity 'r[octet.evidence_codebase_index.query.lookup]'
```

Render a static review page from the same model:

```sh
cargo octet evidence-index codebase html --index artifacts/octet/evidence-codebase-index.json --output artifacts/octet/evidence-codebase-index.html
```

The generated index is Octet review and provenance infrastructure. It proves artifact identity and linkage only.

It does not prove correctness, semantic equivalence, or standalone Valence release ownership.
