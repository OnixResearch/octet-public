## Phase 1: Diagnostic foundation

- [x] Create `src/diagnostics.rs` with `LintDiagnosticMeta` inventory and `emit_tiger_style_lint()` wrapper
- [x] Declare and wire the new `diagnostics` module from `src/lib.rs` so lint modules can compile against the shared wrapper
- [x] Record principle, guide section, domain group, guidance, and example text policy for the existing 25 lints in one metadata table
- [x] Copy the canonical principle, guide section, domain group, guidance, and example text strings verbatim from the design appendix into the metadata inventory
- [x] Add a unit test proving diagnostic metadata names exactly match `LINT_INVENTORY`
- [x] Add a full metadata snapshot test covering lint name, principle, guide section, domain group, guidance, example mode, and example text
- [x] Assert every `ExampleMode::Required` entry has concrete example text
- [x] Add a shared-helper enforcement test that source-scans lint modules and rejects `Tiger Style (`, `see Tiger Style guide:`, `.help(`, `.note(`, `span_help(`, and `span_note(` outside `src/diagnostics.rs`

## Phase 2: Wire rich diagnostics into lint emissions

- [x] Route every lint emission through `emit_tiger_style_lint()` and remove direct help/note emission from lint modules before any snapshot refresh
- [x] Keep explicit lint/pass registration code as the source of truth for pass wiring; do not introduce a runtime function-pointer registration table while adding the shared inventory
- [x] Bind each lint below to its exact appendix-derived `LintDiagnosticMeta` row; do not paraphrase principle, guide section, domain group, guidance, example mode, or example text during rewiring
- [x] Safety lints: `compound_assertion`, `assertion_density`, `unbounded_loop`, `ignored_result`, `unchecked_narrowing`, `unchecked_division`, `raw_arithmetic_overflow`, `catch_all_on_enum`, `no_unwrap`, `no_panic`, `no_recursion`
- [x] Naming lints: `bool_naming`, `acronym_style`, `numeric_units`, `float_for_currency`
- [x] Structure lints: `function_length`, `compound_condition`, `ambiguous_params`, `platform_dependent_cast`, `unjustified_allow`, `nested_conditionals`
- [x] Purity lints: `ambient_clock`, `contradictory_time`, `verified_purity`
- [x] Concurrency lint: `multi_lock_ordering`

## Phase 3: Migrate to a true tool-lint namespace

- [x] Spike `no_unwrap` as a single-lint `declare_tool_lint!(pub tigerstyle::no_unwrap)` under the current Dylint harness before converting the remaining 24 lints
- [x] In that spike, register a one-member bare compatibility alias for `no_unwrap` and prove bare `disabled_lints`, bare `[tigerstyle.lint_levels]`, and the current bare lint-level path still resolve through it
- [x] Confirm the spike passes all three proof points (Dylint load, tool-scoped UI attr resolution, bare config / lint-level resolution); if any fails, stop and revisit the design before converting the remaining 24 lints
- [x] Save the spike transcript under the change evidence path, showing Dylint load, tool-scoped attr resolution, and bare config resolution for `no_unwrap`
- [x] After the spike passes, convert all 25 existing consumer-facing lint statics from `declare_lint!` to `declare_tool_lint!(pub tigerstyle::...)`
- [x] Add companion safety lint `unjustified_no_panic_allow` so `#[allow(tigerstyle::no_panic)]` remains reviewable after `no_panic` becomes a true tool lint
- [x] Keep the original static lint IDs registered; do not clone `Lint` descriptors and do not replace explicit pass registration with a runtime function-pointer table
- [x] Add helpers that derive canonical `tigerstyle::<lint>` names from the existing bare inventory keys without duplicating the lint matrix in multiple places
- [x] Register canonical domain groups from inventory-derived lists of enabled static lint IDs: `tigerstyle::safety`, `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`, `tigerstyle::concurrency`
- [x] Register one-member bare compatibility aliases for every enabled lint so existing bare `dylint.toml` keys and repo-local bare per-lint fixtures keep resolving without `unknown lint tool: tigerstyle`
- [x] Omit disabled lints from both canonical domain groups and bare compatibility aliases so `disabled_lints` still beats source-level attrs and level overrides
- [x] Add a canonical group-membership snapshot covering all five tool-scoped domain groups and their ordered members
- [x] Add a bare-alias snapshot covering every bare lint name and the tool lint it resolves to

## Phase 4: Verify tool-scoped attrs and compatibility paths

- [x] Add a standalone UI fixture proving `#![allow(tigerstyle::safety)]` suppresses safety lints
- [x] In that same fixture, prove a different-domain naming lint still warns so cross-domain isolation is explicit
- [x] Add a standalone UI fixture proving `#![deny(tigerstyle::naming)]` escalates a naming lint to an error
- [x] Keep canonical tool-group fixtures self-contained with `#![feature(register_tool)]`, `#![register_tool(tigerstyle)]`, and the existing `--edition=2024` UI path
- [x] Keep the existing bare per-lint config fixtures passing so `[tigerstyle.lint_levels] no_unwrap = ...` continues to work without renaming config keys
- [x] Preserve `no_panic_todo_allow` semantics under the tool-lint migration by routing invalid `#[allow(tigerstyle::no_panic)]` escapes through `unjustified_no_panic_allow`
- [x] Keep `no_panic_todo_allow` as the dedicated UI proof for justified item/block `todo!()` escapes staying quiet while unjustified or non-`todo!()` uses warn via `unjustified_no_panic_allow`
- [x] Keep existing bare allow-by-default fixture paths working where they are intentionally exercising compatibility aliases rather than canonical tool-scoped attrs
- [x] Add a deterministic `lint -> emitting fixture` coverage manifest for all 26 lints and test that it matches the metadata inventory
- [x] Prove every `lint -> emitting fixture` mapping points to a fixture that actually emits that lint and is not a suppressed/non-emitting case
- [x] Add a deterministic fixture-set artifact listing the full existing UI fixture corpus plus the new canonical tool-group fixtures and compatibility/config fixtures
- [x] Add a test validating that the fixture-set artifact covers the full UI fixture corpus plus the new canonical tool-group fixtures and compatibility/config fixtures

## Phase 5: Docs, snapshots, and end-to-end proof

- [x] Update README, downstream rollout docs, and templates to document canonical `tigerstyle::...` lint/group names while keeping `dylint.toml` keys in bare lint-name form
- [x] Revalidate the archived Aspen starter-rollout reference at `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` and save evidence that the tool-lint migration does not break that consumer path
- [x] State explicitly that standalone raw-rustc tool-scoped attrs require the supported nightly toolchain plus `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- [x] Refresh all existing UI `.stderr` fixtures plus the new canonical tool-group fixtures to the canonical warning/help/note format
- [x] Run `cargo test --lib`
- [x] Run `cargo test --test ui -- --ignored --nocapture`
- [x] Save the `cargo test --test ui -- --ignored --nocapture` transcript under the change evidence path before closing the change
- [x] Verify all fixtures and snapshots pass in `nix flake check`
