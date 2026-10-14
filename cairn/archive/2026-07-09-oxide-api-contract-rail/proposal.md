## Why

Octet emits policy, receipt, artifact, and standards surfaces that other tools consume. The Oxide API/tooling stack around Dropshot, Typify, Progenitor, Drift, OpenAPI linting, and API manager workflows shows a useful pattern: generate machine contracts from typed service surfaces, lint the generated contracts, diff compatibility over time, and generate clients only after those gates pass.

Octet should adapt that pattern for its own checked path and evidence APIs without adopting hidden middleware, non-Nickel human config, or unbounded compatibility claims.

## What Changes

- Define an API/schema contract rail for Octet's public machine surfaces.
- Require typed source-of-truth DTOs and Nickel-generated or Rust-derived contracts before clients or downstream fixtures rely on a surface.
- Add compatibility diffing for generated schemas, OpenAPI-like documents, receipt contracts, and command JSON outputs.
- Add positive and negative corpora for accepted changes, breaking changes, malformed documents, and overclaiming generated clients.
- Preserve functional-core / imperative-shell boundaries: pure compatibility decisions operate over in-memory contract inputs; CLI shells own file reads and report rendering.

## Impact

- Downstream consumers get earlier warnings for contract drift.
- Octet's evidence and policy surfaces become easier to test through generated clients/fixtures.
- Existing human-authored Nickel policy remains authoritative where humans compose config; generated schema artifacts stay development/release evidence, not a replacement for typed Rust DTOs.
