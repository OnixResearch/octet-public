## Why

Octet checks Rust source, policy, dependency, proof, and external-evidence surfaces, but it does not yet provide a first-class checked path for WebAssembly modules/components emitted by that source. Aspen and future Kamacite/Animus consumers need deterministic artifact facts such as validation result, component world, imports/exports, enabled proposals, memory/table declarations, producer metadata, and exact byte identity before runtime admission.

A dedicated wasm-tools-backed artifact rail is needed so each consumer does not write a different parser, feature policy, finding vocabulary, or Valence export.

## What Changes

- Add a Nickel-authored Wasm artifact profile registry with deterministic generated runtime input.
- Use one pinned `wasm-tools` crate cohort directly for validation, WIT/component parsing, metadata inspection, and deterministic test generation.
- Validate modules and components from exact bytes without executing them.
- Keep exact artifact BLAKE3 identity separate from normalized WIT/interface identity and reject semantic-equivalence overclaims.
- Emit structured findings and an artifact bundle covering feature posture, imports/exports, world conformance, memories/tables/growth, metadata, limits, and policy results.
- Provide a stable exact-byte CLI/report handoff so Mantle can invoke the rail after build-local validation and bind the independent result into component materialization bundles without reimplementing Octet policy.
- Add positive and negative corpora using checked fixtures plus `wasm-smith`, `wit-smith`, `wasm-mutate`, and `wasm-shrink` where appropriate.
- Export only recorded-only or boundary evidence to Valence.

## Impact

- **Surfaces**: standards policy registry, artifact verification CLI, pure analysis core, generated contracts/manifests, Wasm/WIT fixtures, evidence bundles, and Valence export.
- **Consumers**: Mantle is the primary production orchestrator and binds reports into materialization bundles; Aspen, Kamacite, Animus, OnixOS, and ChaosControl may independently reverify the same stable facts without transferring runtime/build semantics to Octet.
- **Claims**: passing proves bounded syntax/profile/source-shape facts only. It is not execution, sandbox, authority, provenance truth, semantic equivalence, or release eligibility.
