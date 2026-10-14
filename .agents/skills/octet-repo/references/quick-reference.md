# Octet Repo Quick Reference

Compact map for common work in `/path/to/git/octet` (Octet repo checkout).

## Always-start commands

Run Rust work inside nix devshell:

```bash
nix develop -c bash -c 'cargo check'
```

When shared target-dir lock contention appears:

```bash
tmp_target="$(mktemp -d /tmp/octet-target.XXXXXX)"
trap 'rm -rf "$tmp_target"' EXIT
CARGO_TARGET_DIR="$tmp_target" nix develop -c bash -c 'cargo test --lib'
```

## Most useful verification commands

### General

```bash
nix develop -c bash -c 'cargo check'
nix develop -c bash -c 'cargo test --lib'
nix flake check
```

### UI harness

```bash
nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'
nix develop -c bash -c 'BLESS=1 cargo test --test ui ui -- --ignored --nocapture'
nix develop -c bash -c 'cargo test --test ui autofix_machine_applicable -- --ignored --nocapture'
```

### cargo-octet engine

```bash
nix develop -c bash -c 'cargo check -p cargo-octet'
nix develop -c bash -c 'cargo test -p cargo-octet'
nix build .#cargo-octet
```

## Area -> first files to read

### New or changed lint

Read in this order:
1. `src/lib.rs`
2. `src/diagnostics.rs`
3. `src/config.rs` if configurable
4. `src/<domain>/<lint>.rs`
5. one nearby lint in same domain
6. `tests/fixtures/ui_fixture_set.json`
7. `tests/fixtures/lint_emitting_fixtures.json`
8. target fixture in `ui_tests/`

Then keep these in sync:
- lint implementation
- lint registration in `src/lib.rs`
- diagnostic metadata in `src/diagnostics.rs`
- `ui_tests/*.rs` and `.stderr` / `.fixed`
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`

### Diagnostics / groups / help text

Read:
- `src/diagnostics.rs`
- `src/lib.rs`
- `README.md`
- relevant `.stderr` fixtures under `ui_tests/`

### Config / thresholds / per-lint levels

Read:
- `src/config.rs`
- `templates/dylint.toml`
- `docs/downstream-rollout.md`
- `README.md`

Config contract:
- Rust attrs use canonical tool names like `octet::no_unwrap`
- `dylint.toml` uses bare keys like `no_unwrap`
- `disabled_lints` wins over `lint_levels`

### UI fixture work

Read:
- `tests/ui.rs`
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- target fixture `.rs`
- target `.stderr` or `.fixed`
- fixture-local `Cargo.toml` and `dylint.toml` when present

UI facts:
- fixture selection happens inside `tests/ui.rs`
- canonical ignored rail: `cargo test --test ui ui -- --ignored --nocapture`
- autofix rail: `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`
- subdir fixture `ui_tests/foo/foo.rs` snapshots to `ui_tests/foo.stderr`
- tool-scoped attrs need `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`

### Downstream / rollout / consumer docs

Read:
- `docs/downstream-rollout.md`
- `templates/dylint.toml`
- `README.md`
- `flake.nix`
- archived evidence paths cited by docs before changing claims

Current consumer path:
- consumer `dylint.toml`
- consumer `[workspace.metadata.octet]`
- `cargo octet check` or `nix run .#cargo-octet -- check`

Historical wrapper path is replay-only.

### OpenSpec

Read:
- `openspec/templates/tasks.md`
- target change under `openspec/changes/`
- target main spec under `openspec/specs/`
- `AGENTS.md`

Rules:
- task files use `I<n>` / `V<n>` / `H<n>` plus `## Verification Coverage`
- main spec files stay full spec form, not delta form
- cite archived Aspen references, not Aspen HEAD, for rollout history

## File-path map

### Core repo files

- `README.md` -> user-facing usage and repo overview
- `Cargo.toml` -> workspace root
- `rust-toolchain.toml` -> pinned nightly toolchain
- `flake.nix` -> devshell, checks, consumer-check helper
- `AGENTS.md` -> repo-local agent guidance
- `.agent/napkin.md` -> repo-local mistakes and working patterns

### Lint implementation by domain

- `src/safety/`
  - `compound_assertion.rs`
  - `ignored_result.rs`
  - `no_unwrap.rs`
  - `no_panic.rs`
  - `no_todo.rs`
  - `no_recursion.rs`
  - `unchecked_narrowing.rs`
  - `unchecked_division.rs`
  - `unbounded_loop.rs`
  - `catch_all_on_enum.rs`
  - `explicit_defaults.rs`
  - `unbounded_channel.rs`
  - `unbounded_collection_growth.rs`
  - `assertion_density.rs`
  - `raw_arithmetic_overflow.rs`
  - `sentinel_fallback.rs`
- `src/naming/`
  - `acronym_style.rs`
  - `bool_naming.rs`
  - `negated_predicate.rs`
  - `numeric_units.rs`
  - `float_for_currency.rs`
- `src/structure/`
  - `function_length.rs`
  - `nested_conditionals.rs`
  - `platform_dependent_cast.rs`
  - `usize_in_public_api.rs`
  - `too_many_parameters.rs`
  - `compound_condition.rs`
  - `unjustified_allow.rs`
  - `ambiguous_params.rs`
- `src/purity/`
  - `ambient_clock.rs`
  - `verified_purity.rs`
  - `contradictory_time.rs`
- `src/concurrency/`
  - `multi_lock_ordering.rs`

### Registration / metadata / config

- `src/lib.rs` -> lint inventory, registration, groups, compatibility aliases
- `src/diagnostics.rs` -> help text, examples, group snapshots, suggestion helpers
- `src/config.rs` -> `dylint.toml` loading and validation
- `src/test_context.rs` -> test-context detection helpers

### Runner / integration

- `cargo-octet/src/main.rs`
- `cargo-octet/src/cli.rs`
- `cargo-octet/src/engine.rs`
- `cargo-octet/src/config.rs`

### UI harness / snapshots

- `tests/ui.rs`
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- `ui_tests/*.rs`
- `ui_tests/*.stderr`
- `ui_tests/*.fixed`

### Consumer rollout docs

- `docs/downstream-rollout.md`
- `templates/dylint.toml`
- `verus/README.md`

## Gotchas

- Repo ignores `/.pi`; tracked repo-local skills belong under `.agents/skills/`
- Global skill names can collide; repo-local skill uses distinct name `octet-repo`
- `todo!()` escape hatches belong to `no_todo`, not `no_panic`
- If diagnostic examples/help text change, expect `.stderr` snapshot churn
- If you add a lint but skip fixture manifests, repo ends up half-registered
- Aspen rollout claims must point at archived evidence paths when making starter-profile statements
