## Contract

- [x] [serial] Define the dependency-direction policy and its three invariants: core depends only on core + std, adapters depend inward on core, core never depends on adapters. r[octet.dependency_direction.invariants]
- [x] [serial] Define direct-vs-transitive edge classification and the distinct diagnostic for transitive leakage. r[octet.dependency_direction.edges]
- [x] [serial] Define the dependency-edge scope (normal + build vs dev) as a configurable boundary field. r[octet.dependency_direction.scope]
- [x] [serial] Depends on `policy-driven-crate-boundaries` for declared core/shell/adapter crate sets. [depends:policy-driven-crate-boundaries]

## Implementation

- [x] [serial] Add a pure `dependency_direction` core function classifying edges from a metadata graph and three crate sets. r[octet.dependency_direction.invariants]
- [x] [serial] Add an imperative shell running `cargo metadata`, classifying workspace packages, and mapping violating edges to diagnostics. r[octet.dependency_direction.shell]
- [x] [serial] Distinguish direct and transitive violations in the diagnostic message and remediation. r[octet.dependency_direction.edges]
- [x] [serial] Wire the check into `check_architecture` behind the architecture family selection. r[octet.dependency_direction.invariants]

## Verification

- [x] [serial] Add pure-core property tests for the three invariants over synthetic graphs (positive and negative). r[octet.dependency_direction.invariants]
- [x] [serial] Add a checked-in `cargo metadata` fixture driving the shell without invoking cargo. r[octet.dependency_direction.shell]
- [x] [serial] Add workspace fixtures for clean, core-to-adapter, core-to-external-infrastructure, and shell-outward edges. r[octet.dependency_direction.edges]
- [x] [serial] Add a transitive-leakage fixture proving the distinct diagnostic fires. r[octet.dependency_direction.edges]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.dependency_direction.invariants]
