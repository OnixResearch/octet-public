## Why

Octet's `octet-standards` architecture checker hardcodes the FCIS core/shell layout: core paths (`crates/tigerstyle-core/`), the std shell (`crates/tigerstyle-std/`), the forbidden core dependencies (`tokio`, `tracing`, ...), and the external-domain leakage patterns (`axum::`, `sqlx::`, `tonic::`, ...). A consumer whose layout is `domain/`, `application/`, `infrastructure/`, `adapters/` — or `crates/billing-core` + `crates/billing-http` — cannot use the FCIS, capability-injection, or external-leakage checks at all. The `CapabilityApiObligations` already reads `core_source_scopes` from policy, but the no_std, forbidden-dependency, shell-depends-on-core, and external-leakage checks ignore policy and match hardcoded strings.

This blocks hexagonal/DIP enforcement on any project that is not laid out exactly like the octet repo. Nickel is the repo's designated owner for human-authored boundary config, so the boundary scopes and pattern tables belong there.

## What Changes

- Add a `crate-boundary` mechanism kind to the policy registry so consumers declare `core_crates`, `shell_crates`, `adapter_crates`, `forbidden_core_deps`, and `external_domain_patterns` in Nickel.
- Plumb every architecture check that currently reads a hardcoded literal through the policy mechanism, falling back to the existing octet-layout defaults only when no policy entry is present.
- Export the boundary config as a generated policy artifact covered by the existing config-receipt-family freshness checks.
- Keep the default profile producing identical diagnostics to today for the octet repo, so no behavior changes for in-repo consumers.

## Impact

- **Files**: `standards/policies/contracts.ncl`, `standards/policies/registry.ncl`, `crates/octet-standards/src/model.rs`, `crates/octet-standards/src/architecture.rs`, `crates/octet-standards/src/artifacts.rs`, `crates/octet-standards/src/lib.rs`, new positive/negative fixtures under `standards/profiles/`, docs under `docs/standards/`.
- **Testing**: positive fixtures for a custom-layout consumer passing and failing each generalized check; negative fixtures rejecting malformed boundary config; a regression test proving the octet default profile is byte-identical to the pre-change diagnostics; `cairn validate --root .` and Cairn gates for this change.
