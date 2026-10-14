# Aspen Tigerstyle Rollout Notes

## Target Workspace

- Workspace: `/path/to/git/aspen`
- Landed Aspen commit: `909e5e9fa` (`tigerstyle: add phase-1 rollout path and follow-up changes`)
- Check path: `scripts/tigerstyle-check.sh`
- Workspace tuning: `dylint.toml`
- Follow-up Aspen changes:
  - `openspec/changes/tigerstyle-phase-2-coordination-cleanup/`
  - `openspec/changes/tigerstyle-phase-3-safety-debt-cleanup/`

## Why The Check Path Is A Script, Not A Flake Check

Aspen's nix-managed nightly toolchain exposes two Dylint integration problems:

1. `rustup show active-toolchain` reports `nix-managed`, which breaks Dylint's
   toolchain-specific driver and library naming unless the check wraps `rustup`
   and `cargo`.
2. current `dylint_driver` still needs the nightly fix from
   `trailofbits/dylint@0e0a71eefe6f01563d11acc6e1d7af1d505934a9`.

`scripts/tigerstyle-check.sh` handles both without changing the caller's shell:

- uses a temporary `CARGO_HOME/config.toml` patch for `dylint_driver`
- wraps `rustup`, `cargo`, and `dylint-link`
- points Dylint at the local tigerstyle repo via `TIGERSTYLE_PATH` or `../tigerstyle`
- defaults to a low-noise pilot package set, `--lib`, and `--no-deps`

This is the current equivalent CI/check path for Aspen. A true flake check can
follow once `cargo-dylint` packaging and the nix-managed toolchain story stop
requiring wrappers.

## Initial Starter Lint Set

Phase 1 starter lints:

- `ambient_clock`
- `compound_assertion`
- `contradictory_time`

These are the families we can expose first without forcing Aspen to swallow the
much larger safety-debt categories from the baseline triage.

## Phase 1 Pilot Packages

Default pilot scope in `scripts/tigerstyle-check.sh`:

- `aspen-time`
- `aspen-hlc`
- `aspen-core`

Reason:

- they exercise real production code
- they keep the first end-to-end run small enough to be repeatable
- they avoid the immediate test-noise and legacy debt seen in larger crates

## Deferred Lints And Packages

### Deferred lint families

Deferred from phase 1 because baseline triage is still too noisy:

- `ignored_result` — 299 matches
- `no_unwrap` / `expect` — 421 matches
- `no_panic` — 323 matches
- `unchecked_narrowing` — 510 matches
- `unbounded_loop` — 173 matches

These need targeted cleanup or narrower scoping before they belong in the
starter path.

### Deferred package scope

`aspen-coordination` is intentionally not in the default pilot set yet.
A focused lib-only, no-deps run still produced:

- 21 `ambient_clock` warnings
- 2 `compound_assertion` warnings
- 4 `no_panic` warnings

That is actionable, but too noisy for the phase-1 default path. It should move
into phase 2 after a dedicated cleanup pass.

## Promotion Criteria

Promote a deferred lint family or package only when all of these hold:

1. the scripted check runs end to end without ad-hoc shell setup beyond the
   documented requirements
2. the candidate rollout scope has a bounded finding set with a clear owner
3. any threshold, allowlist, or explicit exception is recorded with rationale
4. no known tigerstyle false positive is being papered over with blanket allow
5. the rollout notes are updated with the new scope and remaining deferrals

## `dylint.toml` Rationale

Committed Aspen tuning:

- `function_length_max = 100` — phase 1 should not fail immediately on larger
  orchestration handlers while decomposition work is still deferred
- `nesting_max_depth = 4` — coordinator/RPC code still needs one extra level
  during the first rollout
- `assertion_density_min_lines = 30` — ignore thin shell wrappers for now
- `assertion_density_min_assertions = 2` — keep the default assertion count once
  the function is large enough to matter
- `acronym_allowlist = ["IO"]` — explicit, documented Aspen exception for
  transport/storage vocabulary

## Validation Summary

### Landed downstream state

- Aspen rollout landed in commit `909e5e9fa`
- Aspen working tree was clean immediately after that commit
- targeted touched-crate validation passed:

```bash
cd /path/to/git/aspen
cargo clippy -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination --lib -- -D warnings
```

- both follow-up Aspen OpenSpec changes validate:
  - `tigerstyle-phase-2-coordination-cleanup`
  - `tigerstyle-phase-3-safety-debt-cleanup`

### Default pilot run

Command:

```bash
cd /path/to/git/aspen
scripts/tigerstyle-check.sh
```

Observed warnings:

- `aspen-time`: 2 `ambient_clock`
- `aspen-core`: 4 `ambient_clock`
- `aspen-hlc`: 0 starter-lint findings

Total starter-path findings: 6 warnings, all in `ambient_clock`.

### Deferred-package spot check

Command:

```bash
cd /path/to/git/aspen
scripts/tigerstyle-check.sh -p aspen-coordination -- --lib
```

Observed warnings:

- 21 `ambient_clock`
- 2 `compound_assertion`
- 4 `no_panic`

Result: package intentionally deferred from the default phase-1 path.
