## Context

`crates/octet-standards/src/architecture.rs` carries the boundary identity as module-level `const` strings: `CORE_LIB_PATH_SUFFIX`, `CORE_MANIFEST_PATH_SUFFIX`, `STD_MANIFEST_PATH_SUFFIX`, `CORE_CRATE_DEPENDENCY`, `CORE_FORBIDDEN_DEPENDENCIES`, `EXTERNAL_DOMAIN_TYPE_PATTERNS`. The `CapabilityApiObligations` struct proves the pattern works: it already pulls `forbidden_ambient_apis`, `core_source_scopes`, and `shell_constructor_scopes` from the policy mechanism and falls back to `FALLBACK_*` constants only when the policy field is empty. The remaining checks have not been generalized.

The policy registry already supports mechanism kinds `crate-boundary`, `capability-api`, `bounded-primitive`, and `type-state`. Today only the `crate-boundary` kind is used to assert the octet core/shell pair exists, not to parameterize the per-check scopes and patterns.

## Approach

1. Extend the `crate-boundary` mechanism contract with: `core_crates` (path globs / manifest suffixes), `shell_crates`, `adapter_crates`, `forbidden_core_deps`, `external_domain_patterns`, and an optional `core_no_std` flag. Keep the existing fields backward compatible.
2. Introduce a `CrateBoundaryObligations` struct mirroring `CapabilityApiObligations`, built from the policy mechanism with `FALLBACK_*` constants preserving today's octet layout.
3. Route `check_core_no_std`, `check_core_forbidden_dependencies`, `check_std_shell_owns_std`, `check_shell_depends_on_core`, and `check_external_domain_leakage` through the new obligations instead of hardcoded suffixes.
4. Export the resolved boundary config in the generated policy artifact so config-receipt freshness covers it.
5. Author a default `octet-layout` profile in Nickel that reproduces today's literals exactly, and a `custom-layout` example profile demonstrating a `domain/` + `infrastructure/` consumer.

## Risks

- Existing octet-repo diagnostics must not change. The default profile and a byte-identical regression test guard this.
- Path-glob semantics differ from exact suffix matching. Restrict matching to manifest-path suffixes and `contains`-style source-path segments that the current code already uses, to avoid a glob engine dependency.
- A consumer listing no `crate-boundary` mechanism must get today's default behavior, not silence. The fallback must be the octet defaults, never an empty obligation set.

## Validation

- Contract tests accept a complete `crate-boundary` mechanism and reject missing `core_crates`, unknown mechanism fields, and empty non-default profiles.
- A custom-layout positive fixture passes all generalized checks and a negative fixture fails each one (missing no_std, forbidden dep present, shell missing core dep, external leak).
- A regression test asserts the default profile produces diagnostics byte-identical to a saved baseline captured before the change.
- `nix run path:/path/to/git/cairn#cairn -- validate --root .` and the three Cairn gates pass.
