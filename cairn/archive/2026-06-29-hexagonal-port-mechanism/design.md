## Context

The architecture standard already describes the anti-corruption layer as "define a trait for what you need; implement it for the external type; never let external types leak into your domain" (`docs/standards/source/architecture.adoc`). That is a port, but the checker has no first-class representation of it. The `CapabilityApiObligations` struct is the closest existing analogue and provides the pattern for a new `PortObligations` struct that reads declared ports from policy and falls back to no-op when none are declared.

Source-shape detection of `trait` declarations and `impl Trait for ...` blocks is feasible with `rustc` HIR, but `octet-standards` is deliberately a source-shape checker (not a Dylint lint) and already tokenizes `pub` chunks. A lightweight trait-name + `impl ... for` text scan in declared scopes is consistent with the existing checker's fidelity.

## Approach

1. Add a `port` mechanism kind to the Nickel contract: `port_traits` (qualified names), `core_owner_scope`, `permitted_adapter_scopes`, and optional `forbidden_concrete_types` patterns.
2. Add a `PortObligations` struct and three checks: `check_ports_defined_in_core`, `check_port_has_adapter_impl`, `check_core_no_concrete_port_impl`.
3. Source-scan declared core scope for `trait <Name>` declarations and adapter scopes for `impl <Name> for` blocks; flag a declared port missing from core, a declared port with no adapter impl, and a core-scope `impl <Port> for <ConcreteInfraType>`.
4. Treat an undeclared `trait` as out of scope (no diagnostic) so the mechanism stays opt-in.

## Risks

- Text-based trait detection can false-positive on macro-generated or re-exported traits. Scope the scan to declared core/adapter source paths and to non-macro `trait`/`impl` lines, mirroring the existing `public_api_chunks` fidelity.
- Declared ports with intentionally no adapter (e.g. a port reserved for a future adapter) would falsely fail. Allow a `reserved: true` annotation per port that suppresses the missing-impl diagnostic while keeping the core-purity checks active.
- Over-gating real projects. Default the mechanism to opt-in with an empty port set so consumers without declared ports see no change.

## Validation

- Contract tests accept a complete `port` mechanism and reject a port with no `core_owner_scope`.
- Positive fixture: one port in core, one adapter impl, no core concrete impl — clean.
- Negative fixtures: port missing from core; port with no adapter impl; core `impl Port for ReqwestClient`.
- Opt-in no-op fixture: a consumer with no `port` mechanism produces zero port diagnostics.
- `nix run path:/path/to/git/cairn#cairn -- validate --root .` and the three Cairn gates pass.
