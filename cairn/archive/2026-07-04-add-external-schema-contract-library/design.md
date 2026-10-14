## Context

External evidence rails such as SARIF, dependency audit reports, dependency trust reports, and future tool receipts are not Octet-native DTOs at their ingestion boundary. Today those inputs can be parsed by Rust code, but the repository lacks a shared generated-contract library that records the upstream schema, generator identity, and fixture coverage for each accepted external shape.

`json-schema-to-nickel` demonstrates how JSON Schema can become Nickel contracts, with lazy record contracts when possible and predicate-style fallbacks when needed. `nickel-schemastore` demonstrates how many generated contracts can be packaged behind a stable Nickel index. Octet should adapt both patterns locally with stricter freshness and no-network validation.

## Decisions

### 1. Generate from pinned local schemas

**Choice:** External schema contracts are generated only from checked-in schema inputs with recorded dialect metadata and BLAKE3 digests.

**Rationale:** Release evidence must be replayable offline. Pinning schemas avoids treating mutable upstream schema URLs as evidence inputs.

### 2. Package contracts as an indexed library

**Choice:** Generated contracts live under a stable `standards/` contract bundle with an indexed entry point, plus a manifest that names source schema, generated artifact, generator command, BLAKE3 digests, and byte lengths.

**Rationale:** The `nickel-schemastore` pattern makes generated contracts discoverable without forcing each consumer to know generated filenames.

### 3. Preserve generator capability metadata

**Choice:** Each generated contract record names whether the generator produced an inspectable record contract, a predicate fallback, or a mixed contract, and records unsupported JSON Schema features encountered.

**Rationale:** Generated predicate contracts can validate data but are less useful for inspection and documentation. Octet should not hide that distinction from reviewers.

### 4. Keep Octet DTO boundaries intact

**Choice:** External schema contracts validate external input shapes only. Octet-emitted receipts continue to be owned by Rust DTOs with generated or registered Nickel metadata where applicable.

**Rationale:** This prevents a generated third-party schema from becoming the normative source for Octet receipt compatibility.

## Risks / Trade-offs

- **Schema drift:** Upstream tools can change output without changing local pinned schemas. Mitigate by requiring explicit schema refresh and fixture updates.
- **Dialect mismatch:** JSON Schema dialects beyond supported generator capabilities can produce incomplete validation. Mitigate with unsupported-dialect fail-closed behavior.
- **False confidence:** A valid external report shape does not prove the external tool's analysis is correct. Mitigate with explicit non-claims in receipts and docs.
- **Large generated files:** Contract bundles can be noisy. Mitigate with deterministic generation and manifest freshness checks.

## Validation

- Contract generation is stable across repeated runs from the same checked-in schema inputs.
- Valid external evidence fixtures pass their generated contracts.
- Negative fixtures for missing required fields, wrong field types, unsupported schema versions, and malformed reports fail with field-identifying diagnostics.
- A stale generated contract or stale manifest fails policy freshness validation.
- Cairn validate and gates pass for this change.
