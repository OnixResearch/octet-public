## Context

`cargo metadata --format-version 1` yields the resolve graph with per-package dependencies. `octet-standards` already invokes cargo and parses `Cargo.toml` for the FCIS checks, so a metadata-backed graph check is a natural extension. The `policy-driven-crate-boundaries` change introduces declared `core_crates`, `shell_crates`, and `adapter_crates` sets, which this check needs to classify nodes.

Today's `check_core_forbidden_dependencies` only reads a core crate's own `[dependencies]` table. It misses workspace path dependencies on adapter crates and transitive runtime leaks.

## Approach

1. Add a pure `dependency_direction` core function: given a metadata resolve graph and three classified crate sets, return the set of violating edges with a direction label (`core_to_adapter`, `core_to_external_infrastructure`, `shell_outward_to_adapter`).
2. Add an imperative shell that runs `cargo metadata`, classifies each workspace package against the declared `crate-boundary` sets, and maps violating edges to policy diagnostics.
3. Distinguish direct edges (a core manifest lists an adapter) from transitive edges (a core reaches an adapter through a non-core intermediate).
4. Keep the existing forbidden-dependency string check as a fast pre-filter; the graph check is the authoritative DIP gate.

## Risks

- `cargo metadata` execution is an effect and must stay in the shell, not the verified core. The graph classification itself is pure and stays in core.
- A large workspace graph could be slow. The check should only walk workspace members and their direct + one-hop transitive edges, not the entire external resolve tree.
- Build-script and dev-dependency edges are not architecture dependencies. The check should restrict to normal + build dependencies or be configurable per boundary set.

## Validation

- Pure-core property tests over synthetic graphs prove the three invariants for direct and transitive edges.
- A `cargo metadata` fixture (checked-in JSON) drives the shell without invoking cargo in tests.
- Positive and negative workspace fixtures cover clean, core→adapter, core→external-infrastructure, and shell-outward cases.
- `nix run path:/path/to/git/cairn#cairn -- validate --root .` and the three Cairn gates pass.
