## Contract

- [x] [serial] Define the `port` mechanism kind: `port_traits`, `core_owner_scope`, `permitted_adapter_scopes`, `forbidden_concrete_types`, and `reserved` flag. r[octet.hexagonal_ports.mechanism]
- [x] [serial] Define the opt-in default: a consumer with no `port` mechanism MUST produce zero port diagnostics. r[octet.hexagonal_ports.opt_in]
- [x] [serial] Define the three port invariants: ports defined in core, each non-reserved port has an adapter impl, core has no concrete-infrastructure port impl. r[octet.hexagonal_ports.invariants]
- [x] [serial] Depends on `policy-driven-crate-boundaries` for core/adapter scope resolution. [depends:policy-driven-crate-boundaries]

## Implementation

- [x] [serial] Extend Nickel `MechanismContract` and `registry.ncl` with the `port` mechanism kind and fields. r[octet.hexagonal_ports.mechanism]
- [x] [serial] Add `PortObligations` and the three checks (`check_ports_defined_in_core`, `check_port_has_adapter_impl`, `check_core_no_concrete_port_impl`) to `architecture.rs`. r[octet.hexagonal_ports.invariants]
- [x] [serial] Honor the `reserved` flag by suppressing the missing-impl diagnostic while keeping core-purity checks active. r[octet.hexagonal_ports.invariants]
- [x] [serial] Wire the port checks into `check_architecture` behind the architecture family selection. r[octet.hexagonal_ports.invariants]

## Verification

- [x] [serial] Add a positive fixture: port in core, adapter impl present, no core concrete impl — clean. r[octet.hexagonal_ports.invariants]
- [x] [serial] Add negative fixtures for port-missing-from-core, missing-adapter-impl, and core-concrete-port-impl. r[octet.hexagonal_ports.invariants]
- [x] [serial] Add an opt-in no-op fixture proving a consumer with no `port` mechanism produces zero port diagnostics. r[octet.hexagonal_ports.opt_in]
- [x] [serial] Add a reserved-port fixture proving the missing-impl diagnostic is suppressed but core-purity checks still fire. r[octet.hexagonal_ports.invariants]
- [x] [serial] Add negative contract tests rejecting a port mechanism with no `core_owner_scope`. r[octet.hexagonal_ports.mechanism]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.hexagonal_ports.opt_in]
