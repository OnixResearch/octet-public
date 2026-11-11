# Compiler-generated expansion admission

## Cause

A pure workspace failed architecture gate mode on compiler-generated imports and test harnesses. Plain Cargo accepted all targets. The collector required a macro definition for every expanded span, although compiler AST passes are not macros.

The diagnostic control identifies `AstPass(StdImports)` and `AstPass(TestHarness)` in the consumer. The ten-unit owner fixture also identifies `AstPass(ProcMacroHarness)`.

## Boundary

The collector recognizes exactly those three compiler enum variants. Their expansion facts, generated spans, and call-site spans remain present. Their optional `macro_definition` remains `None`. The collector does not invent a macro definition or skip generated bodies, calls, or effects.

Every other definition-less expansion still emits `UnsupportedExpansion`. Unit controls reject root context and unresolved bang, attribute, and derive macros. This is not a blanket exception for macros or generated source. It does not establish support for every compiler desugaring.

No receipt DTO or serialization format changed. The existing optional macro field correctly describes non-macro compiler passes. Producer classification changed, so consumers must bind the new collector identity and regenerate their observations.

## Evidence

The real Cargo fixture failed before the correction. After the correction, its ten units retained complete coverage and no unsupported expansions. Its effect observations and missing-shard rejection remained intact.

A separate consumer contains a pure `no_std` library and a real host package. Both packages include normal and test compilation units. With the correction, gate mode reports `clean`, with no findings or unknowns. Independent artifact verification reports `valid`.

A mutation adds a dormant `std::fs::read` function to the core. Plain Cargo still accepts all targets. The architecture gate rejects the function with `domain-core-effect` and exit 1. Changing the policy without regenerating its export rejects before compilation. The fixture never executes its filesystem function.

The ignored real-Cargo forged-question fixture copies the selected target workspace, then has a procedural macro emit a *different* filesystem-reading `?` expression with every generated token stamped with an authored input `?` span. The forged function's item and resolved `std::fs::read` Call/Filesystem Effect appear at the apparently authored location. Rustc's `Try::branch` lowering at that location has a distinct expanded span; its Expansion links back to the stolen source span without a macro definition. The collector records `UnsupportedExpansion` on that exact lowered span, and Gate mode retains open `required-compiler-unknown` and `domain-core-effect` findings for the actual forged unknown and effect in the same selected production unit. The test checks these identities and their source-unit binding; a separate directly authored `?` cannot satisfy the forged assertions.

This controls the observed `?` lowering, not arbitrary source-token authenticity. It adds no exemption, policy change, waiver, or effect-closure assertion.

Exact owner checks:

```sh
nix build .#checks.x86_64-linux.compiler-architecture-targets .#cargo-octet --no-link -L
cargo test -p octet -p cargo-octet -p octet-architecture-ir --lib -- --test-threads=1
cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings
cargo fmt --all -- --check
```

Cargo commands use the pinned Nix shell, empty compiler wrappers, and disabled incremental compilation. Owner-only evidence is under `~/.local/state/workday-h7/admission-r15/`. The red control, corrected compiler control, scoped suites, pure consumer, effect mutation, and stale export each have separate logs.

## Non-claims

This bounded control does not complete Workday H7. The Workday source-role matrix, full lint catalog, compatible feature rows, and production entry points still need separate acceptance. No tenant calls occurred.
