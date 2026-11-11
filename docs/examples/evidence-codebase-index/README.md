# Evidence codebase index example

Build a deterministic local Octet-owned index over an existing `cargo octet check` bundle. The index goes under `target/`, outside the bundle that `cargo octet artifact verify` checks and outside any directory a later `--artifact-dir` scan reads:

```sh
cargo octet check --output-format json --artifact-dir artifacts/octet
cargo octet evidence-index codebase build --artifact artifacts/octet/results.jsonl --artifact artifacts/octet/provenance.jsonl --output target/octet-evidence-index/evidence-codebase-index.json
```

A check with no findings writes an empty `results.jsonl`; drop that `--artifact` in that case, because strict mode reports an empty file as an unsupported artifact family.

Query by a stable identity:

```sh
cargo octet evidence-index codebase query --index target/octet-evidence-index/evidence-codebase-index.json --identity octet.runner.artifact-bundle
```

Render a static review page from the same model:

```sh
cargo octet evidence-index codebase html --index target/octet-evidence-index/evidence-codebase-index.json --output target/octet-evidence-index/evidence-codebase-index.html
```

The generated index is Octet review and provenance infrastructure. It proves artifact identity and linkage only.

It does not prove correctness, semantic equivalence, or standalone Valence release ownership.
