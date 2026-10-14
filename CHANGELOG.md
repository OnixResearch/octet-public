# Changelog

Notable changes that affect consumers. Organized by date since consumers pin to
specific commits via `TIGERSTYLE_DYLINT_REV`.

For the full lint catalog, see `docs/lint-reference.md`.
For rollout profiles, see `docs/downstream-rollout.md`.

## 2026-10-14: Historyless signed collaboration fixtures

The public-source candidate replaces both Cairn COB bundles with real signed
import/update and conflict/recovery observations rooted in independent empty Git
commits. The conflict manifest uses schema v2 and `synthetic_base_git_oid`;
historical Octet ancestry and its old manifest field are not retained.

Repository-owned replay scripts reproduce the scenarios with public fixture
seeds and the exact Cairn and Nickel cohorts. Shell-measured transport records
have separate Nickel sources and registry families; Cairn stdout remains
unmodified. Passing fixture checks establish bounded local integration and
bundle/export integrity, not publication, distributed convergence, or release
readiness.

## 2026-08-26: Compiler-visible authorized effect protocols

Project architecture policies can now classify outbound effect families as protected or unprotected. Protected protocols name exact authority inputs, decisions, opaque witnesses, executors, observations, receipts, use cardinality, source scopes, and non-claims.

Activated collection records selected type shapes, constructor safety, and bounded optimized-MIR provenance. Strict modes block public or replayable witnesses, undeclared or unsafe constructors, missing executor witness parameters, unguarded effect sites, unknown origins, and receipt promotion without an approved success observation. Passing evidence does not prove credential validity, revocation freshness, actor trust, external success, or receipt authenticity.

## 2026-08-19: Integrated project architecture gate

An activated policy now makes `cargo octet check` evaluate the resolved Cargo graph and compiler Architecture IR. Lint and architecture phases keep separate statuses, findings, evidence tiers, receipts, and non-claims. Input-integrity failures block every active mode. Policy findings block only `gate` and `admission`.

`admission` now binds the `deterministic-core` profile and one workspace-relative effect-closure policy. Octet runs strict lint, architecture evaluation, and closure in order. It emits separate linked receipts and rejects denied closure or tampered policy, graph, compiler, provider, command, and parent-status identities. Unconfigured projects keep their existing output bytes and emit no architecture or closure artifacts.

## 2026-08-12: Compiler-backed Architecture IR

Activated project policies now collect bounded rustc facts into content-addressed shards. Octet emits normalized Architecture IR, coverage, issue, and receipt sidecars only for selected production compilation units. Incomplete or stale evidence fails closed and keeps explicit non-claims.

## 2026-07-27: Opt-in project architecture policy

Projects can now select a typed Nickel FCIS and Hexagonal policy through `[workspace.metadata.octet].architecture_policy`. Activation verifies workspace-relative paths, the deterministic JSON export, contract identity, evaluator identity, and the BLAKE3 freshness manifest before Cargo runs.

Projects without the pointer retain existing lint and strict-profile behavior. They receive no project architecture diagnostics, receipt, or architecture-clean claim. Start with `inventory` mode and read `docs/project-architecture-policy.md` before promotion.

## 2026-07-26: Machine-readable assurance profiles

Octet now packages a closed Nickel assurance profile for the active OnixOS lifecycle proof chain. The profile links owners, protected properties, assumptions, trusted-base elements, required evidence, validation commands, and explicit non-claims across construction, semantic-design, verification, effect-control, and provenance-readiness planes.

`nix build .#octet-assurance-profile-onixos-lifecycle` exposes the authored profile, contract, generated JSON, and Nickel Export BLAKE3 manifest. `checks.x86_64-linux.assurance-profile-onixos-lifecycle` validates exact generation plus positive and negative fixtures. The profile declares requirements only; it does not report evidence success, cumulative proof, physical readiness, or release eligibility. See `docs/assurance-profiles.md`.

## 2026-07-15: Profile-bound Clippy restrictions

`cargo octet evidence collect --rail clippy-co-reporting --run` now accepts the embedded Nickel-authored `production-safety`, `canonical-determinism`, and `shell-boundary-inventory` restriction profiles through `--profile`; `--config PATH` may supply another checked catalog. Profiled runs construct fixed workspace/all-target Clippy argv, reject arbitrary commands, parse only Clippy JSON stdout, and bind full-catalog plus selected-profile BLAKE3 identities into the receipt.

The catalog deliberately cherry-picks restrictions and rejects the blanket `clippy::restriction` group. Clippy remains optional `static_lint` evidence complementary to Octet's semantic lints; passing does not prove behavioral correctness, panic freedom, memory safety, runtime determinism, compiler correctness, formal proof, whole-program correctness, or release eligibility. See `docs/clippy-restriction-profiles.md`.

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
