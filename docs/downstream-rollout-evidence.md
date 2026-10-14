# Downstream rollout evidence

Historical evidence, Aspen-specific replay data, and archived wrapper
documentation. This is the companion file to `downstream-rollout.md`, which
contains active rollout guidance.

New consumers should start with `consumer-guide.md` and the active profiles in
`downstream-rollout.md`. This file exists for replay, audit, and promotion
decisions.

## Reference state: use the Aspen archive, not Aspen HEAD

Phase-1 reference state lives in the archived OpenSpec change:

- Aspen rollout commit: `909e5e9fa` (`octet: add phase-1 rollout path and follow-up changes`)
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- rollout notes: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- default-run transcript: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- deferred-package spotcheck: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-coordination-spotcheck.txt`
- phase-1 counts: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
- wrapper + config snapshot: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`

Current Aspen workspace scripts may already include later packages and later lint
families. Reconstruct the starter path from the archived change above, not from
whatever Aspen HEAD happens to run today.

## Current widened reference: Aspen phase 2

Use phase 1 as the canonical starter path. After that, Aspen's widened phase-2
reference is archived in this repo at:

- replay helper: `bash openspec/changes/archive/2026-04-14-aspen-phase2-widening/evidence/replay-aspen-phase2.sh`
- Aspen HEAD snapshot: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/evidence/aspen-phase2-meta.txt`
- widened default pilot: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/evidence/aspen-phase2-default-pilot.txt`
- targeted `aspen-coordination` rerun: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/evidence/aspen-phase2-coordination.txt`
- widening assertions: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/evidence/aspen-phase2-check.txt`

What that widened reference proves:

- default scope now includes `-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination`
- Aspen's current default lint set is now `ambient_clock`,
  `compound_assertion`, `contradictory_time`, `ignored_result`, and
  `no_unwrap`
- a targeted `aspen-coordination` rerun no longer emits starter-lint spans

Treat this as the first post-starter promotion example, not as a replacement
for the archived phase-1 starter reconstruction.

## Generated policy artifact dogfood

The first Aspen-backed smoke for generated policy artifacts is saved at
`docs/examples/aspen-policy-artifact-dogfood.md`.

What it proves:

- `nix run .#octet-policy-artifacts -- <out-dir>` emits the expected
  generated files.
- `octet-standards check --policy-artifacts-dir <out-dir>` can run against
  Aspen source without reading the Nickel registry directly.
- The selected Aspen production roots (`aspen-core`, `aspen-coordination`, and
  `aspen-rpc-handlers`) report zero `embedded` diagnostics while still exposing
  structural policy-evaluation metadata for
  `octet.embedded.bounded-queues`.
- The same artifact-backed seam is now packaged for downstream flakes as
  `lib.mkPolicyArtifactConsumerCheck` and regression-covered by
  `checks.x86_64-linux.policy-artifact-consumer-fixture`.
- The bounded-queue structural policy is enforced from the generated artifact
  fields. A downstream scratch mutation that introduces a configured unbounded
  constructor is expected to fail the focused policy-artifact consumer check.

## Aspen starter references for boundary-first profile

- historical phase-1 baseline: 6 warnings total, all `ambient_clock`
  - evidence: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- promoted starter replay on the same package set: 10 warnings total
  - 6 `ambient_clock`
  - 1 `explicit_defaults`
  - 3 `usize_in_public_api`
  - evidence: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-default.txt`

## Aspen classification and promoted starter families

Saved evidence now splits into two layers:

- archived classification pass for the six new families:
  - replay helper: `bash openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/replay-aspen-new-lints.sh`
  - summary table: `openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/validation-counts.txt`
- promoted starter replay for the copied starter profile:
  - replay helper: `bash openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/replay-aspen-promoted-starter.sh`
  - config snapshot: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-config.toml`
  - summary table: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-counts.txt`
  - saved spotcheck: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-coordination.txt`

Low-noise rule for promotion stayed the same:

- `starter-candidate`: <= 3 findings on the starter scope and <= 5 findings on
  `aspen-coordination`
- `inventory-first`: anything larger stays out of starter-profile promotion for
  now

Current Aspen-backed starter status:

| Family | Starter scope | `aspen-coordination` | Starter status |
| --- | ---: | ---: | --- |
| `explicit_defaults` | 1 | 2 | `promoted` |
| `unbounded_channel` | 0 | 0 | `promoted` |
| `unbounded_collection_growth` | 0 | 8 | `inventory-first` |
| `usize_in_public_api` | 3 | 0 | `promoted` |
| `too_many_parameters` | 0 | 15 | `inventory-first` |
| `negated_predicate` | 0 | 2 | `promoted` |

The promoted starter replay stays pinned to the documented starter package set
from `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`:
`aspen-time`, `aspen-hlc`, and `aspen-core`.

Only `unbounded_collection_growth` and `too_many_parameters` still stay out of
the copied starter-profile config among these six families.

## Aspen phase-1 config values

Aspen phase-1 example values were:

- `function_length_max = 100`
- `nesting_max_depth = 4`
- `assertion_density_min_lines = 30`
- `assertion_density_min_assertions = 2`
- `acronym_allowlist = ["IO"]`

Those values belong in Aspen's rollout notes and `dylint.toml`, not in
`octet` defaults.

Evidence:

- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`

## Historical wrapper path

The copied shell-wrapper path is retained only inside archived replay materials
and historical rollout reconstruction. It is no longer the active recommended
consumer path.

For historical replay, use the archived reconstruction helpers directly instead
of copying a live repo wrapper.

That historical wrapper owned these responsibilities:

- derives a full nightly toolchain triple from `octet/rust-toolchain.toml`
  and the host triple
- wraps `rustup show active-toolchain` and `rustup which rustc` when the local
  toolchain manager reports something nonstandard like `nix-managed`
- wraps `cargo` and `dylint-link` so `RUSTUP_TOOLCHAIN` is set during build and
  link steps
- bootstraps an optional pinned `dylint_driver` in an isolated temporary crate
  so the consumer workspace `Cargo.lock` stays untouched
- restores the original `Cargo.lock` and fails if the check path still mutates
  the consumer lockfile
- points Dylint at a local `octet` checkout through `TIGERSTYLE_PATH` or a
  sibling checkout
- owns the documented starter profile, default scope, and default cargo-check
  args in one reviewable script

The starter-profile lint list in the template is documentation and review
scoping, not a hard per-lint CLI filter. External Dylint runs still use the
consumer's enabled warn-by-default octet lints unless the consumer changes
source-level lint configuration.

Minimum assumptions for the template:

- `bash`, `cargo`, `rustc`, `python3`, `cargo-dylint`, `dylint-link`, and
  `git` are on `PATH`
- standard Unix userland is available for `awk`, `basename`, `chmod`, `cmp`,
  `cp`, `ln`, `mkdir`, `mktemp`, and `sed`
- the consumer repo has its own `dylint.toml`
- a local `octet` checkout exists next to the consumer repo, or
  `TIGERSTYLE_PATH` is set
- the maintainer edits `default_scope` before landing the script

When archive-only wrappers are unnecessary, delete the wrapper section and call
the supported first-class runner instead:

- `cargo octet check`
- `nix run path:/path/to/octet#cargo-octet -- check`

Do not treat plain `cargo dylint` as the rollout surface. The template is a
recipe, not a policy requirement.

### Aspen wrapper facts worth preserving

Aspen phase 1 needed three wrapper behaviors:

1. `rustup show active-toolchain` had to return a real nightly triple instead of
   `nix-managed`
2. `dylint-link` needed `RUSTUP_TOOLCHAIN` during link steps
3. `dylint_driver` needed an unreleased upstream fix; the current template
   satisfies that by bootstrapping a pinned driver in an isolated temporary
   crate before touching the consumer workspace

That third point is exactly why the template exposes `TIGERSTYLE_DYLINT_REV`
instead of baking Aspen's patch revision into `octet` itself.

## Second consumer proof

Archived non-Aspen replay proof in this repo:

- replay helper: `bash openspec/changes/archive/2026-04-14-second-consumer-proof/evidence/replay-second-consumer.sh`
- consumer snapshot: `openspec/changes/archive/2026-04-14-second-consumer-proof/evidence/second-consumer-meta.txt`
- run transcript: `openspec/changes/archive/2026-04-14-second-consumer-proof/evidence/second-consumer-proof.txt`
- replay assertions: `openspec/changes/archive/2026-04-14-second-consumer-proof/evidence/second-consumer-check.txt`

Current saved proof uses `aspen-verus-metrics` from a disposable sandbox. That
archived helper still copies the historical shell wrapper plus
`templates/dylint.toml`, edits `default_scope` to `-p aspen-verus-metrics`, then
runs the copied script through a octet-provided nightly toolchain
environment. Keep this as replay evidence only; new consumers should use
`cargo octet check` plus `[workspace.metadata.octet]`.

## Aspen phase-1 reconstruction

Exact saved-transcript replay path from this repo:

```bash
bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh
```

That archived capture helper writes
`openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`.
It prints the same top-level command preamble saved in that transcript, then
invokes the replay helper below. The replay helper performs these steps
internally:

1. create a disposable sandbox parent with Aspen's required sibling layout:
   - `aspen/` worktree at commit `909e5e9fa`
   - `aspen-dns -> /path/to/aspen-dns`
   - `aspen-wasm-plugin -> /path/to/aspen-wasm-plugin`
2. copy the archived phase-1 shell wrapper together with `templates/dylint.toml`
   into the Aspen sandbox checkout
3. configure the Aspen phase-1 scope:
   - `-p aspen-time`
   - `-p aspen-hlc`
   - `-p aspen-core`
4. apply the archived Aspen phase-1 `dylint.toml` values
5. export `TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9`
   while running the default path and the `aspen-coordination` spotcheck
6. confirm the archived phase-1 result:
   - 6 warnings total
   - `aspen-time`: 2 `ambient_clock`
   - `aspen-core`: 4 `ambient_clock`
   - `aspen-hlc`: 0 starter findings
7. confirm the archived defer decision:
   - 21 `ambient_clock`
   - 2 `compound_assertion`
   - 4 `no_panic` (historical pre-split count; `todo!()` debt now lives under `no_todo`)

Aspen needs the sibling entries because path dependencies in `aspen-dns` and
`aspen-wasm-plugin` resolve back through `../aspen`. The helper also removes the
sandbox worktree on exit.

Archive references for that reconstruction:

- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-coordination-spotcheck.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
- `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`
