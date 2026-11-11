## Why

Octet now has bounded source-token Function Object v1 receipts, replay checks, compatibility fixtures, expanded internal corpus coverage, fresh-target determinism, CI gates, and fresh-build receipt parity. The next semantic-quality step is to define how compiler-grounded HIR/MIR evidence can supplement those objects without turning hashes into correctness or semantic-equivalence claims.

## What Changes

- **HIR supplemental evidence contract**: Define a toolchain-scoped optional evidence layer that can record compiler-derived item identity, dependency observations, and macro/context caveats alongside Function Object v1.
- **MIR supplemental evidence contract**: Define a later optional evidence layer for MIR-derived observations with explicit rustc-version and optimization/context caveats.
- **Boundary preservation**: Keep source-token object identity as the stable v1 review/provenance surface and require all HIR/MIR supplements to preserve `claims_correctness = false` and `claims_semantic_equivalence = false`.
- **Verification plan**: Require deterministic contract metadata/docs/tests before any implementation claims HIR/MIR support.

## Capabilities

### New Capabilities
- `hir-mir-object-evidence-contract`: Specification for future compiler-grounded object evidence supplements.

### Modified Capabilities
- `standards-codification`: Extends Valence/Octet object evidence policy with HIR/MIR supplement boundaries.

## Impact

- **Files**: OpenSpec change under `openspec/changes/archive/2026-05-19-hir-mir-object-evidence-contract/`.
- **APIs**: No runtime API change in this scaffold.
- **Dependencies**: No dependency changes in this scaffold.
- **Testing**: Validate the OpenSpec change and all specs with strict OpenSpec validation plus `git diff --check`.
