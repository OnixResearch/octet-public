---
name: octet-repo
description: Work in the Octet repo at /path/to/git/octet. Use when adding or changing lints, diagnostics, config, UI fixtures, cargo-octet engine code, downstream rollout docs/templates, or OpenSpec evidence in this workspace.
compatibility: Project-specific skill for /path/to/git/octet (Octet repo checkout). Assumes nix devshell, nightly rustc_private tooling, Dylint, and OpenSpec.
---

# Octet Repo

Use `references/quick-reference.md` for a compact command sheet, file-path map, and area-to-file lookup.

## Skill command usage

Direct command form:

```bash
/skill:octet-repo <area> [name]
```

Areas:
- `lint <lint-name>`
- `runner`
- `config`
- `ui [fixture-or-lint]`
- `downstream`
- `openspec [change-name]`

Examples:

```bash
/skill:octet-repo lint no_unwrap
/skill:octet-repo runner
/skill:octet-repo ui compound_assertion
/skill:octet-repo downstream
```

Interpret first arg as target area. Second arg, when present, is the lint, fixture, or change name to inspect first.

## First pass

Before editing:
- read `AGENTS.md`
- read `README.md`
- read `Cargo.toml`
- read `src/lib.rs`
- read files in the target area

Always run Rust commands inside the nix devshell here. Plain `cargo` outside the shell can miss rustc-private and OpenSSL/pkg-config setup.

Canonical shell form:

```bash
nix develop -c bash -c 'cargo check'
```

If shared target-dir lock contention shows up, use a temporary `CARGO_TARGET_DIR` for targeted verification.

## Workspace map

- `src/lib.rs` -> lint inventory, registration, canonical groups, compatibility aliases
- `src/safety/` -> safety lints
- `src/naming/` -> naming lints
- `src/structure/` -> structure lints
- `src/purity/` -> purity lints
- `src/concurrency/` -> concurrency lints
- `src/diagnostics.rs` -> lint metadata, group membership, guidance text, suggestion helpers
- `src/config.rs` -> `dylint.toml` parsing, defaults, validation, per-lint levels
- `cargo-octet/src/` -> first-class downstream runner and engine
- `ui_tests/` -> checked-in UI fixtures and `.stderr` / `.fixed` artifacts
- `tests/ui.rs` -> custom UI harness and autofix rails
- `tests/fixtures/` -> fixture inventory snapshots and coverage manifests
- `templates/dylint.toml` -> consumer-owned config template
- `docs/downstream-rollout.md` -> rollout profiles, Aspen references, promotion playbooks
- `openspec/` -> change specs, archived rollout evidence, task templates
- `verus/` -> small formal-spec area

## Area guides

### `lint`

Use for adding a new lint or changing an existing one.

Read first:
- `src/lib.rs`
- `src/diagnostics.rs`
- `src/config.rs` if thresholds or per-lint levels matter
- target lint file under `src/<domain>/<lint>.rs`
- one nearby lint in same domain
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- target fixture under `ui_tests/`

When adding or extending a lint, keep this checklist in sync:

1. implement lint code under the right domain module
2. register it in `src/lib.rs`
3. add diagnostic metadata in `src/diagnostics.rs`
4. add or update UI fixture coverage in `ui_tests/`
5. update `tests/fixtures/ui_fixture_set.json`
6. update `tests/fixtures/lint_emitting_fixtures.json`
7. if lint is configurable, add a `*_custom/` fixture with local `Cargo.toml` + `dylint.toml`
8. if lint emits structured fixes, add or update `.fixed` artifacts and autofix coverage

Repo rules worth keeping straight:
- new lints use canonical tool names in Rust attrs, like `octet::no_unwrap`
- `dylint.toml` keeps bare lint keys like `no_unwrap`
- compatibility aliases are deliberate; keep legacy source attrs and bare-name behavior working unless a change explicitly removes them
- `todo!()` escape hatches belong to `no_todo`, not `no_panic`

Useful commands:

```bash
nix develop -c bash -c 'cargo check'
nix develop -c bash -c 'cargo test --lib'
nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'
nix develop -c bash -c 'cargo test --test ui autofix_machine_applicable -- --ignored --nocapture'
```

Refresh checked-in UI snapshots with:

```bash
nix develop -c bash -c 'BLESS=1 cargo test --test ui ui -- --ignored --nocapture'
```

### `runner`

Use for `cargo-octet` engine changes, downstream check-path logic, lockfile fail-closed behavior, embedded source extraction, or CLI work.

Read first:
- `cargo-octet/src/main.rs`
- `cargo-octet/src/cli.rs`
- `cargo-octet/src/engine.rs`
- `cargo-octet/src/config.rs`
- `README.md` usage section
- `docs/downstream-rollout.md`
- `flake.nix` if Nix consumer path changes

Keep first-class engine path intact:
- `cargo octet check`
- `nix run .#cargo-octet -- check`

Do not regress back to copied historical wrapper guidance unless work is explicitly about archived replay paths.

Useful commands:

```bash
nix develop -c bash -c 'cargo check -p cargo-octet'
nix develop -c bash -c 'cargo test -p cargo-octet'
nix build .#cargo-octet
```

### `config`

Use for `dylint.toml` parsing, defaults, per-lint levels, template changes, or lint group behavior.

Read first:
- `src/config.rs`
- `templates/dylint.toml`
- `README.md` config section
- `docs/downstream-rollout.md`
- `src/diagnostics.rs` if groups or canonical names change

Keep these contracts straight:
- source attrs and group attrs use canonical tool names like `octet::safety`
- `disabled_lints` and `[octet.lint_levels]` use bare lint names
- unknown lint names and invalid levels warn and are ignored
- `disabled_lints` wins over per-lint levels

Useful commands:

```bash
nix develop -c bash -c 'cargo test --lib config'
nix develop -c bash -c 'cargo test --lib diagnostics'
```

### `ui`

Use for UI fixture coverage, snapshot churn, standalone fixture Cargo manifests, or autofix proof.

Read first:
- `tests/ui.rs`
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- target fixture `.rs`
- target `.stderr` or `.fixed`
- target fixture-local `Cargo.toml` / `dylint.toml` when present

Repo-specific UI rules:
- canonical ignored UI rail is `cargo test --test ui ui -- --ignored --nocapture`
- autofix rail is `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`; it compares `.fixed`, reruns `cargo fix` for idempotence, and checks the fixed crate warning-free
- fixture selection happens inside `tests/ui.rs`; do not tack on a random extra libtest filter expecting one fixture to run
- tool-scoped fixture attrs need `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- fixture subdirs like `ui_tests/foo/foo.rs` still snapshot to `ui_tests/foo.stderr`

### `downstream`

Use for consumer onboarding docs, rollout profiles, templates, Aspen replay references, or Nix consumer-check helpers.

Read first:
- `docs/downstream-rollout.md`
- `templates/dylint.toml`
- `README.md` usage and rollout sections
- `flake.nix` consumer helper section when Nix path changes
- archived evidence paths mentioned in the docs if you touch claims or counts

Current recommended consumer path:
- consumer-owned `dylint.toml`
- `[workspace.metadata.octet]` in consumer `Cargo.toml`
- `cargo octet check` or `nix run ...#cargo-octet -- check`

Historical wrapper scripts are replay-only. Keep docs explicit about that.

### `openspec`

Use for spec changes, rollout evidence, task reshaping, or archive work in this repo.

Read first:
- `openspec/templates/tasks.md`
- target change under `openspec/changes/`
- relevant main spec under `openspec/specs/`
- `AGENTS.md` OpenSpec notes

Repo-local rules:
- task files use `I<n>` / `V<n>` / `H<n>` structure plus `## Verification Coverage`
- `I<n>` tasks are implementation-only; evidence capture belongs in `V<n>` or `H<n>`
- main spec files under `openspec/specs/*/spec.md` stay in full spec form, not delta form
- for rollout history, cite archived Aspen references, not Aspen HEAD

## Verification

Default targeted checks:

```bash
nix develop -c bash -c 'cargo check'
nix develop -c bash -c 'cargo test --lib'
nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'
```

When full repo proof matters:

```bash
nix flake check
```

Good package-targeted commands:

```bash
nix develop -c bash -c 'cargo check -p cargo-octet'
nix develop -c bash -c 'cargo test -p cargo-octet'
```

## Repo-specific gotchas

- This repo may still coexist with legacy global `slotcar` guidance. Project-local repo guidance uses the distinct `octet-repo` skill name so pi loads the Octet-specific instructions.
- Use the nix devshell for Rust commands here.
- `cargo test --test ui ui -- --ignored --nocapture` is the right ignored UI invocation here.
- `dylint.toml` uses bare lint names; Rust source attrs use canonical `tigerstyle::...` names.
- Adding a lint without updating `src/diagnostics.rs`, `tests/fixtures/ui_fixture_set.json`, and `tests/fixtures/lint_emitting_fixtures.json` leaves the repo in a half-registered state.
- If a change touches user-visible diagnostic metadata or examples, expect `.stderr` snapshot churn.
- Use archived Aspen rollout evidence when making claims about starter profiles or promoted families.
