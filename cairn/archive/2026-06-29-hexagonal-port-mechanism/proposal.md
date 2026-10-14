## Why

Hexagonal architecture's defining feature is **ports**: traits declared in core, implemented by adapters, with core never `impl`ing a port for a concrete infrastructure type. Octet has capability-injection detection (ambient-authority checks) but no notion of a port. Today a "port" is just any `trait`, indistinguishable from an internal abstraction. There is no check that an adapter implements a declared port, no check that core keeps ports abstract, and no check that core never binds a concrete infrastructure type to a port's interface.

Making ports explicit and machine-checkable turns the ports-and-adapters shape from an implicit reviewer judgment into a structural gate. It depends on `policy-driven-crate-boundaries` to know core vs adapter scopes, and composes with `dependency-direction-graph-check` (a port `impl` in an adapter is what authorizes the adapter→core edge).

## What Changes

- Add a `port` mechanism kind to the policy registry: consumers declare port trait paths, the core scope that owns them, and the adapter scopes permitted to implement them.
- Add a `port` check to `octet-standards` that verifies: ports are declared as traits in core; at least one adapter `impl`s each declared port; core never `impl`s a port for a concrete infrastructure type.
- Report missing-port-implementation (a declared port with no adapter), port-defined-in-shell (a port trait outside core), and core-concrete-port-impl (core binding a port to an infrastructure type).
- Keep the mechanism opt-in so consumers without declared ports are unaffected.

## Impact

- **Files**: `standards/policies/contracts.ncl`, `standards/policies/registry.ncl`, `crates/octet-standards/src/model.rs`, `crates/octet-standards/src/architecture.rs`, `crates/octet-standards/src/lib.rs`, new fixtures, `docs/standards/source/architecture.adoc`. Depends on `policy-driven-crate-boundaries`.
- **Testing**: positive fixture (port in core, adapter impl present, no core concrete impl); negative fixtures for each of the three violations; opt-in no-op fixture for a consumer with no ports; `cairn validate --root .` and Cairn gates.
