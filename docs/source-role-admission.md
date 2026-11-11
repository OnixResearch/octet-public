# Source-role admission

The architecture evaluator currently owns decisions at the package level. It cannot safely apply a different source role inside that package.

A Workday consumer control exposed the gap. A host package declared its entry point as `composition-root` and a child file as `domain-core`. The child contained this dormant function:

```rust
pub fn hidden_read(path: &str) -> bool {
    std::fs::read(path).is_ok()
}
```

The policy permitted filesystem execution at the composition root, not in the core. Revision `8012a7fd8fa394c2e712570943a2e1a88b0f3f0b` compiled all four selected normal/test units. It reported a clean architecture receipt with no findings. Artifact verification also accepted the bundle. These results did not establish the declared source boundary.

## Fail-closed rule

For each declared source role, the gate finds the selected item definitions in that scope. Their package role must match the source role. A type use or implementation does not transfer its source ownership to the package that defines the type. A conflict produces `source-role-unit-mismatch` with integration severity. Inventory and advisory modes cannot turn this unsupported interpretation into acceptance.

The gate retains package-owned decisions. It does not infer a new role or erase the source declaration. Matching package and source roles remain valid. The change does not alter the policy or receipt schema.

A source role of `test` or `generated` also cannot override a different role on a selected package. An excluded source that has no selected item definitions does not establish coverage.

## Remaining capability

Mixed-role packages need a separate fact-level ownership design across effects, ports, constructors, inward types, unknowns, and authority protocols. Until that design has direct positive and negative evidence, callers must keep roles aligned or retain the blocker. This rejection does not prove complete source coverage, nonempty core selection, or a complete feature matrix.
