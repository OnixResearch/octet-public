## Context

The `wasm-tools` repository provides production parsing and manipulation crates including `wasmparser`, `wit-parser`, `wit-component`, `wasm-metadata`, `wasm-smith`, `wit-smith`, `wasm-mutate`, and `wasm-shrink`. The crates are released as one compatibility cohort and may change API across cohort upgrades.

Octet should own a checked artifact profile and evidence shape, not a Wasm runtime. Runtime determinism, capability authorization, component invocation, and release admission remain consumer responsibilities.

## Decisions

### 1. Wasm artifacts use a dedicated standards rail

**Choice:** Add a first-class artifact-analysis rail behind Octet's standards/artifact command surfaces rather than a Dylint pass. The rail reads already-loaded bytes in its analysis core and emits the same stable exit classes and artifact-verification behavior as other Octet evidence bundles.

**Rationale:** Binary/WIT validation is not Rust HIR linting and should not be forced through rustc-private execution.

### 2. Human policy is Nickel-authored

**Choice:** Define profile schemas, allowed artifact kinds, component worlds, enabled/disabled proposals, import/export policies, memory/table/growth rules, metadata posture, and severity promotion in typed Nickel. Export deterministic Rust DTO input and register the config/receipt family in the policy registry and artifact freshness manifest.

**Rationale:** Profile review and merging benefit from Nickel contracts; runtime findings remain Rust-owned outputs.

### 3. One wasm-tools cohort is pinned and identified

**Choice:** Pin the direct library cohort as one reviewed dependency unit and record crate/tool versions plus a BLAKE3 cohort identity in every report. Upgrades require full fixture replay and compatibility review.

**Rationale:** Independently upgrading parser, WIT, metadata, encoder, or generator crates can create mismatched feature and component semantics.

### 4. Parsing and policy decisions are pure

**Choice:** A pure core accepts bytes, normalized profile DTOs, and optional expected WIT facts; it parses, validates, extracts facts, evaluates policy, and returns ordered findings and report DTOs. The CLI shell opens paths, loads config, writes bundles, and renders output.

**Rationale:** Artifact behavior should be testable from byte slices without standing up Cargo, Wasmtime, filesystem fixtures, or subprocesses.

### 5. Byte identity and interface identity are distinct

**Choice:** Record exact module/component BLAKE3 over original bytes. Separately record normalized WIT package/world/import/export identity over a declared canonical representation. Never use interface identity as a substitute for byte identity or as proof that two implementations behave the same.

**Rationale:** Multiple different components can implement the same world, and custom sections/metadata can change bytes without changing interface shape.

### 6. Findings cover bounded artifact facts

**Choice:** The first profile covers binary/component validation, allowed Wasm proposals, module-versus-component classification, WIT world conformance, imports/exports, WASI/host import allowlists, memory/table declarations and growth operations, start/entrypoint posture, producer metadata, size/resource declarations, and malformed or duplicate metadata.

**Rationale:** These facts are actionable before runtime and are directly available from wasm-tools parsers.

### 7. Generated adversarial fixtures are deterministic

**Choice:** Fixture manifests bind generator/mutator identity, seed, input artifact, transform sequence, expected issue class, and optional shrink predicate. Generated cases are checked into or reproducibly materialized from bounded inputs. Every promoted check has positive and negative fixtures.

**Rationale:** Mutation breadth is valuable only when failures can be reproduced and minimized.

### 8. Valence exports preserve bounded roles

**Choice:** Export exact artifact facts and policy decisions as recorded-only or boundary Evidence IR. Do not request property/proof roles or claim execution, authority, sandboxing, semantics, or release readiness.

### 9. Mantle orchestrates production artifact checks

**Choice:** Expose a stable CLI/report contract that accepts exact final bytes plus an explicit Octet profile and emits a self-verifiable bundle. Mantle invokes this contract after build-local validation and binds the report, artifact BLAKE3, profile, and Octet cohort into its materialization bundle. Octet remains independently runnable, does not trust Mantle paths or build success, and does not interpret Mantle runtime or release policy.

**Rationale:** Mantle is the correct shell for reproducible build-stage orchestration, while Octet remains the single owner of static artifact policy and finding semantics.

## Functional core / imperative shell split

- **Pure core**: Wasm/WIT parsing from bytes, fact normalization, profile validation, policy evaluation, finding ordering, identity material construction, fixture expectation evaluation, and report DTO construction.
- **Imperative shell**: path and capability-root handling, Nickel export loading, exact-byte request handling for Mantle or standalone callers, fixture discovery/materialization, artifact-directory writes, optional external tool orchestration, and human/SARIF/JSON rendering.

## Risks / Trade-offs

- Wasm proposals and the Component Model evolve. Cohort pinning and explicit feature profiles limit accidental acceptance drift.
- Mantle build cohorts and Octet parser cohorts can drift independently. Reports bind both exact identities and fail compatibility checks rather than assuming a shared version string.
- Deep mutation corpora can be expensive. Keep deterministic fast and deep lanes with named bounds.
- Producer metadata may be absent or untrusted. Treat it as recorded metadata, never provenance truth.
- Validation cannot establish runtime resource use or hostcall behavior; consumers must enforce runtime boundaries separately.

## Non-Goals

- No execution of Wasm or components from Octet.
- No Wasmtime configuration, WASI host, package registry, component composition, or capability authority.
- No equivalence claim from matching WIT, metadata, or normalized interfaces.
- No replacement for Mantle build evidence, Valence identity semantics, Cairn release policy, or consumer runtime receipts.
- No requirement that standalone Octet analysis run inside Mantle; Mantle is the production orchestrator, not a prerequisite for the pure rail.
