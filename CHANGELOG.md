# Changelog

Notable changes that affect consumers. Organized by date since consumers pin to
specific commits via `TIGERSTYLE_DYLINT_REV`.

For the full lint catalog, see `docs/lint-reference.md`.
For rollout profiles, see `docs/downstream-rollout.md`.

## 2026-07-11: Fail-closed effect closure admission

`cargo octet admit --profile deterministic-core --closure` now runs the existing strict check and then computes conservative source-token effect closure over configured core paths. Selected functions bind deny-mode function addresses; unresolved calls require exact provider/version/source/features/target summaries from typed Nickel policy. Unknown effects, stale summaries, forbidden role edges, and provider drift deny admission.

Admission requires an explicit Cargo `--target TARGET` argument after `--`, preventing ambient Cargo configuration from making the direct check and recorded dependency graph disagree. The new `cargo octet admit verify` command checks stored receipt, complete strict-profile parent, status, summary, policy, dependency-graph, and provenance identities without rerunning Cargo. Closure receipts remain bounded reviewed evidence and do not claim complete name resolution, macro/trait/MIR closure, dependency implementation verification, runtime determinism, whole-program correctness, or release eligibility. See `docs/effect-closure-admission.md`.

## 2026-07-11: Strict Rust profile admission

`cargo octet check --profile <PROFILE>` now selects the Nickel-authored
`robust`, `bounded-systems`, or `deterministic-core` policy. Profiles strengthen
their required lints to `deny`, raise function-address checking to a declared
minimum, and bind the selected profile into BLAKE3 run metadata and provenance.

Unprofiled checks remain on `workspace-metadata`. `deterministic-core` requires
non-empty workspace-relative `core_source_scopes` and a reviewed deny-mode
function-address baseline. These profiles validate bounded direct source shapes;
they are not transitive effect proofs or release certificates. See
`docs/strict-rust-profiles.md`.

The packaged runner now carries its linker closure, so dependency-bearing and
proc-macro consumers can run from `nix run` without inheriting a consumer dev
shell. `concrete_construction_in_core` now classifies only direct constructor
calls and excludes local-domain plus `core`/`alloc`/`std` result types; this
removes duplicate parent-expression and local DTO false positives while
preserving external-constructor findings.

## 2026-05-05: Function fingerprint docs and checks

`cargo octet fingerprint` now has checked operator docs for the full
review lifecycle: human summaries, warning/metadata summaries, script-facing
JSON output, source-scanner boundaries, downstream scratch-consumer behavior,
human/JSON `--diff` parity, and deterministic review manifests. `cargo
octet check` outputs now include stable finding IDs plus replay metadata
for toolchain, config hash, and profile hash. Check runs can now write and gate
against `octet-baseline.json` acceptance files so existing warning-only debt
is explicit while new stable IDs fail CI deterministically.

Use one review command for these executable docs:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

That aggregate gate is also part of `nix flake check`. Focused checks such as
`fingerprint-json-output-docs`, `fingerprint-source-scanner-docs`, and
`fingerprint-docs-downstream-smoke` are intended for local debugging after the
aggregate gate identifies a failing slice.

## 2026-04-16: Starter profile expanded

Promoted four low-noise lint families into the starter rollout profile:
`explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
`negated_predicate`.

Consumers using the starter `disabled_lints` list from `templates/dylint.toml`
will see these lints activate after updating. Evidence and classification data
are in `docs/downstream-rollout-evidence.md`.

## 2026-04-15: `todo!()` moved from `no_panic` to `no_todo`

`no_panic` no longer covers `todo!()`. A new `no_todo` lint owns `todo!()`
detection, and `unjustified_no_todo_allow` requires justification for
`allow(no_todo)` escapes.

**Migration:** consumers with justified `todo!()` stubs under `allow(no_panic)`
must migrate to `allow(no_todo)` or `allow(slotcar::no_todo)`.

## 2026-04-15: Tool-scoped lint attrs

Lint attrs migrated from bare names to tool-scoped canonical names:

- `#[allow(slotcar::no_unwrap)]` instead of `#[allow(no_unwrap)]`
- `#![deny(slotcar::naming)]` for domain groups

Bare names still work through compatibility aliases. Standalone raw-rustc
fixtures need `#![feature(register_tool)]` and `#![register_tool(slotcar)]`.

Config keys in `dylint.toml` stay bare (`no_unwrap`, not `slotcar::no_unwrap`).

## 2026-04-14: First-class runner replaces shell wrapper

`cargo octet check` and `nix run .#cargo-octet -- check` are the
supported consumer entry points. Legacy `cargo slotcar` / `cargo tigerstyle` paths are compatibility aliases. The historical shell wrapper is retained only
in archived replay evidence.

New consumers should use `[workspace.metadata.octet] for runner policy
and `dylint.toml` for lint tuning. See `docs/consumer-guide.md`.

## 2026-04-14: Structured output

`cargo octet check` gained `--output-format` (json, sarif) and
`--artifact-dir` for machine-readable output and saved artifact bundles.

Exit codes: `0` clean/warning-only, `1` error-level findings, `2` integration
failures.

## 2026-04-13: Initial Aspen consumer rollout

First evidence-backed downstream rollout against the Aspen distributed systems
project. Established the starter rollout profile with `ambient_clock` and
`compound_assertion`. Evidence archived in
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`.
