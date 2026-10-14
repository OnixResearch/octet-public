## Why

The current architecture enforcement is string matching: it greps for ambient APIs (`std::env::`, `SystemTime::now(`) and a fixed external-type list. It never inspects the cargo dependency graph, so it cannot catch a core crate depending on `reqwest` *indirectly*, or on a sibling adapter crate, or a shell that depends on the wrong direction. Hexagonal/DIP enforcement needs the dependency graph, which `cargo metadata` already provides for free. `octet-standards` already shells out and parses manifests, so the plumbing exists.

This is the check that proves DIP structurally rather than by source-string heuristic. It depends on the `policy-driven-crate-boundaries` change to know which crates are core, shell, and adapter.

## What Changes

- Add a dependency-direction check to `octet-standards` that consumes `cargo metadata` and the declared `crate-boundary` core/shell/adapter sets.
- Enforce three invariants: core crates depend only on core crates plus std; adapter/infrastructure crates depend inward on core; core crates never depend on adapter crates.
- Report violations as policy diagnostics with the offending edge (dependent crate, dependency crate, direction) and a remediation pointing at the boundary that was crossed.
- Treat transitive leakage as a distinct diagnostic so consumers can distinguish a direct forbidden dep from an indirect one.

## Impact

- **Files**: `crates/octet-standards/src/` (new graph module + wiring in `architecture.rs`), `standards/policies/registry.ncl` (new policy + mechanism field for allowed external dep sets), docs under `docs/standards/source/architecture.adoc`.
- **Testing**: positive fixture with a clean core/shell/adapter graph; negative fixtures for core→adapter, core→infrastructure-external, and shell→wrong-direction edges; transitive-leakage fixture; `cairn validate --root .` and Cairn gates. Depends on `policy-driven-crate-boundaries` for the crate-set declaration.
