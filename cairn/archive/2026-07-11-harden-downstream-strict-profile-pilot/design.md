## Context

`concrete_construction_in_core` currently searches every expression snippet for `::new(` or `::default(`. Parent expressions containing one constructor are therefore reported repeatedly, and source spelling is used as a proxy for type ownership. The accepted requirement instead says to flag external, non-domain, non-`std` construction.

The Nix runner wrapper supplies Cargo, Rust, the Dylint driver, and lint library, but not a linker. Dependency-free smoke tests passed while Valence's `serde_derive` build failed with an unclassified compiler diagnostic.

## Decisions

### 1. Separate pure classification from HIR extraction

**Choice:** Build explicit constructor facts in the lint shell and decide emission in a small deterministic classifier.

**Rationale:** HIR/source-map access remains imperative compiler integration. The policy decision—scope, direct constructor call, test/expansion state, and type ownership—is plain-data logic with positive and negative unit coverage.

### 2. Inspect only direct constructor calls

**Choice:** Match a direct HIR `Call` whose callee path ends in `::new` or `::default`.

**Rationale:** Parent expression snippets must not duplicate one constructor finding. Method calls and arbitrary text containing constructor spellings are not evidence of the target shape.

### 3. Preserve domain and standard value construction

**Choice:** Do not report constructors whose result ADT is local or owned by `core`, `alloc`, or `std`; continue to report external crate ADTs.

**Rationale:** Core code must construct its own DTOs and ordinary in-memory values. Dependency inversion applies to external concrete dependencies, matching the accepted spec. External value types can still be reviewed or avoided by consumers.

### 4. Make the packaged shell self-contained

**Choice:** Add `stdenv.cc` to `cargoOctetRuntimePath`.

**Rationale:** Cargo may need a linker for build scripts and proc macros even when the checked crate is pure. This is packaging authority, not application runtime authority.

### 5. Reproduce the actual downstream boundary

**Choice:** Add a Nix check that invokes the packaged runner against a dependency-bearing Rust fixture without inheriting a consumer dev shell.

**Rationale:** Unit and dependency-free integration tests cannot prove the linker closure is complete.

## Risks / Trade-offs

- Result-type ownership is a bounded HIR heuristic; an external value DTO may still be flagged even when it is not infrastructure.
- Constructor factory functions that do not use `new`/`default` remain outside this lint's claim.
- Adding the linker increases the runtime closure size but removes ambient host-tool dependence.
- The Valence pilot remains a downstream smoke, not a cross-repository release dependency or semantic-correctness claim.
