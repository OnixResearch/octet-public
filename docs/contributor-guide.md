# Contributor guide

This page maps public documentation back to the source files that define the
real contract.

For the landing page and naming table, see `../README.md`.
For consumer onboarding, see `consumer-guide.md`.
For the shipped lint catalog, see `lint-reference.md`.

[Assertion review guide](assertion-review.md)
[Assertion checklist template](../templates/assertion-checklist.md)
[Safety-critical claims guide](safety-critical.md)
[Bounds and pacing review guide](bounds-and-pacing-review.md)
[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)
[Safety spec contract](../openspec/specs/safety/spec.md)
[Resilience review guide](resilience-review.md)
[Five-whys architecture template](../templates/five-whys-architecture.md)
[Performance review guide](performance-review.md)
[Performance sketch template](../templates/performance-sketch.md)
[Performance spec contract](../openspec/specs/performance/spec.md)
[Concurrency and distributed-state review guide](concurrency-state-review.md)
[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)
[Concurrency spec contract](../openspec/specs/concurrency/spec.md)
[System comprehension guide](system-comprehension.md)
[System walkthrough template](../templates/system-walkthrough.md)
[OpenSpec proposal template](../openspec/templates/proposal.md)
[OpenSpec delta-spec template](../openspec/templates/spec.md)
[OpenSpec design template](../openspec/templates/design.md)
[Tiger Style policy registry](../standards/policies/README.md)
[Tiger Style standards contract](standards/policy-contract.md)
[Dynamic evidence profiles](dynamic-evidence-profiles.md)
[Dynamic evidence profile template](../templates/dynamic-evidence-profile.md)
[Rust idiom guidance](rust-idiom-guidance.md)
[Functional API boundary guidance](functional-api-boundaries.md)
[Unison-inspired Rust patterns](unison-inspired-rust-patterns.md)
[Function fingerprint guide](function-fingerprints.md)
[Current verified Valence object evidence](current-head-valence-evidence.md)

## Source-of-truth map

| Contract area | Source of truth |
| --- | --- |
| lint inventory and registration | `src/lib.rs` |
| diagnostic wording, guide sections, domain groups | `src/diagnostics.rs` |
| config schema and defaults | `src/config.rs` |
| cargo subcommand parsing and engine behavior | `cargo-octet/src/cli.rs`, `cargo-octet/src/config.rs`, `cargo-octet/src/engine.rs` |
| standards registry and checker core | `standards/policies/`, `crates/octet-standards/src/` |
| Rust-emitted evidence DTO contracts | Rust field tables in `cargo-octet/src/evidence_contracts.rs`, generated Nickel in `standards/evidence/contracts/`, and fixtures in `standards/evidence/fixtures/` |
| standards source material | `docs/standards/source/` |
| published Nix entry points and helpers | `flake.nix` |
| consumer-owned template | `templates/dylint.toml` |
| landing page and naming map | `README.md` |
| consumer onboarding docs | `docs/consumer-guide.md` |
| lint catalog and config reference | `docs/lint-reference.md` |
| deep rollout playbook | `docs/downstream-rollout.md` |
| OpenSpec documentation contract | `openspec/specs/documentation/spec.md` |
| OpenSpec consumer runner contract | `openspec/specs/consumer-integration/spec.md` |
| UI fixtures and catalog coverage | `ui_tests/`, `tests/ui.rs`, `tests/fixtures/ui_fixture_set.json`, `tests/fixtures/lint_emitting_fixtures.json` |

## If you add or change a lint

Check these source files first:

- `src/lib.rs`
- `src/diagnostics.rs`
- the lint implementation under `src/<domain>/`

Then update the matching public docs and fixtures:

- `docs/lint-reference.md` for name, default level, domain, and summary
- `README.md` if the total lint count or domain summary changes
- `docs/consumer-guide.md` if suppression or rollout guidance changes
- `docs/downstream-rollout.md` if starter or inventory guidance changes
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- the relevant OpenSpec spec (`openspec/specs/documentation/spec.md` and domain specs such as `openspec/specs/safety/spec.md`)

## If you add or change a config key

Check these source files first:

- `src/config.rs`
- any lint implementation that reads the key

Then update:

- `docs/lint-reference.md` config table
- `docs/consumer-guide.md` if onboarding or tuning guidance changes
- `templates/dylint.toml`
- `README.md` if the naming or quick-start surface changes
- the relevant OpenSpec spec (`openspec/specs/documentation/spec.md` and any affected domain spec)

Remember:

- `dylint.toml` keys stay bare (`no_unwrap`)
- source attrs and group attrs stay on the current live namespace (`tigerstyle::no_unwrap`, `tigerstyle::safety`)

## If you add or change standards policy

Check these source files first:

- `standards/policies/registry.ncl`
- `standards/policies/contracts.ncl`
- `crates/octet-standards/src/`
- `standards/check-fixtures/`
- `docs/standards/source/`

Then update:

- `standards/policies/README.md`
- `docs/standards/policy-contract.md`
- `README.md` if command surfaces change
- `flake.nix` checks/apps when verification changes

Keep the seam intact: Dylint lints stay in `src/`; policy registry and
source-shape checks stay in `crates/octet-standards` and `standards/`. If a
Rust type emits persisted evidence or receipts, update the Rust-owned field
source first, regenerate or refresh the Nickel contract, and keep both positive
and negative Nickel fixtures registered in `standards/policies/registry.ncl`.

## If you add or change a published entry point

Check these source files first:

- `cargo-octet/src/cli.rs`
- `cargo-octet/src/engine.rs`
- `flake.nix`

Then update:

- `README.md` naming map and quick-start section
- `docs/consumer-guide.md`
- `docs/downstream-rollout.md` if rollout guidance changes
- `tests/fixtures/consumer-integration/`
- `tests/consumer_integration.rs`
- `openspec/specs/documentation/spec.md`
- `openspec/specs/consumer-integration/spec.md`

If the change touches `TIGERSTYLE_*` names, keep the naming table in
`README.md` as the canonical public list and make deeper docs link back to it
instead of publishing conflicting variants.

## Operational gotchas

These are hard-won lessons that affect daily development in this repo.

### Nix devshell is mandatory

Plain `cargo` outside `nix develop` can miss rustc-private libraries,
OpenSSL/pkg-config, and the pinned nightly toolchain. Always wrap Rust commands:

```bash
nix develop -c bash -c 'cargo check'
```

### Refreshing UI snapshots

When lint diagnostic wording or help text changes, `.stderr` snapshots in
`ui_tests/` will drift. Refresh them with:

```bash
nix develop -c bash -c 'BLESS=1 cargo test --test ui ui -- --ignored --nocapture'
```

### Target directory lock contention

The shared `~/.cargo-target` directory can block on unrelated repo builds. Use a
temporary target directory when this happens:

```bash
tmp_target="$(mktemp -d /tmp/octet-target.XXXXXX)"
trap 'rm -rf "$tmp_target"' EXIT
CARGO_TARGET_DIR="$tmp_target" nix develop -c bash -c 'cargo test --lib'
```

### Tool-scoped attrs in standalone fixtures

Standalone UI fixtures that use current live `tigerstyle::` attrs need:

```rust
#![feature(register_tool)]
#![register_tool(tigerstyle)]
```

Without this, rustc rejects the attrs before the lint can run. Bare lint names
like `#[allow(no_unwrap)]` work without the prelude.

### Staging new files before Nix checks

Git-backed flake sources do not include untracked files. `git add` new fixture
files, workspace members, and standards fixture trees before running
`nix flake check` or `nix develop`.

### Lockfile regeneration after dependency changes

After adding or removing Cargo dependencies, run `cargo generate-lockfile`
before trusting `--locked` Nix builds.

## Verification rails

Run Rust commands inside the Nix devshell here.

Common checks:

```bash
nix develop -c bash -c 'cargo check'
nix develop -c bash -c 'cargo test --lib'
nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'
nix develop -c bash -c 'cargo test -p octet-standards'
nix run .#octet-policy-validate
```

Warning-clean release-readiness checks:

```bash
nix build .#checks.x86_64-linux.warning-clean --no-link -L
nix build .#checks.x86_64-linux.clippy --no-link -L
```

The `warning-clean` rail runs the owned workspace with Rust compiler warnings
denied and covers the `cargo-octet`, `cargo-slotcar`, and `cargo-tigerstyle`
binary targets. The paired `warning-clean-regression` check is a negative
control proving a new dead helper fails under the same warning-deny policy. The
`clippy` rail runs Clippy with `-D warnings -W clippy::pedantic` so pedantic
findings are release-readiness regressions too.

Useful doc-facing spot checks:

```bash
nix develop -c bash -c 'cargo test --test consumer_integration downstream_rollout_docs_point_to_first_class_cli_path -- --nocapture'
rg -n "cargo octet check|cargo-octet|tigerstyle::|\[octet\]|\[workspace.metadata.octet\]" README.md docs/*.md
```

For fingerprint docs/examples, use one review command:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

For Valence object-evidence changes, use one aggregate operator command:

```bash
nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L
```

That aggregate gate composes selected local object corpus replay, compatibility,
caveat taxonomy, named object-corpus fixture freshness, selected THIR fixture
inventory, selected MIR fixture replay, and fresh-build determinism rails. Run
`nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`
whenever a change touches object-corpus inputs such as `cargo-octet/src/object.rs`,
`cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`,
`THIR/MIR fixture inventories`, or `cargo-octet/tests/fixtures/object-corpus/*`; regenerate
stale object-corpus fixtures before closeout. The aggregate includes a named
freshness PASS line,
`valence_object_evidence.object_corpus_fixture_freshness_gate: PASS`. It also includes a named MIR inventory check,
`nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L`,
which is composed into the aggregate as
`valence_object_evidence.mir_supplement_inventory_gate: PASS`. The THIR and MIR
fixture coverage is selected-fixture-only (THIR: `checked-arithmetic`, `call-return`, and `branch-control-flow`; MIR: `checked-arithmetic`, `call-return`, and `branch-control-flow`) and does not imply general THIR or MIR extraction. The aggregate is bounded evidence replay for the
selected corpus; it is not a correctness proof, semantic equivalence proof,
exhaustive Rust coverage proof, or cross-machine/toolchain determinism proof.

For bounded HIR/MIR supplement pilots, emit and replay toolchain-scoped
supplement receipts alongside the canonical Function Object v1 identity:

```bash
cargo octet object supplement emit --family hir --function checked src/lib.rs --output-format json > hir-supplement.json
cargo octet object supplement check --family hir --receipt hir-supplement.json --function checked src/lib.rs --output-format json
cargo octet object supplement emit --family mir --function checked src/lib.rs --output-format json > mir-supplement.json
cargo octet object supplement check --family mir --receipt mir-supplement.json --function checked src/lib.rs --output-format json
cargo octet object supplement emit --family thir --function checked src/lib.rs --output-format json > thir-supplement.json
cargo octet object supplement check --family thir --receipt thir-supplement.json --function checked src/lib.rs --output-format json
cargo octet object corpus contract --output-format json
```

For fixture-backed rustc HIR/THIR/MIR spikes, keep the explicit spike flag in both
emit and replay commands. List the closed selector inventory before choosing a
shape:

```bash
cargo octet object supplement fixtures --family hir --spike rustc-hir-fixture --output-format json
cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture --output-format json
cargo octet object supplement fixtures --family mir --spike rustc-mir-fixture --output-format json
```

The inventory reports each selected checked-in fixture shape, source path,
function name, default marker, replay commands, and explicit non-proof boundary
flags. HIR and THIR may omit `--fixture-shape` to use their default checked
arithmetic fixture. MIR fixture evidence must name `--fixture-shape
checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow` explicitly; missing or
non-selected MIR fixture shapes fail closed:

```bash
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --output-format json > hir-spike-checked.json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --receipt hir-spike-checked.json --output-format json
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --output-format json > hir-spike-branch.json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --receipt hir-spike-branch.json --output-format json
cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic --output-format json > thir-spike-checked.json
cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic --receipt thir-spike-checked.json --output-format json
cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape call-return --output-format json > thir-spike-call-return.json
cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape call-return --receipt thir-spike-call-return.json --output-format json
cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape branch-control-flow --output-format json > thir-spike-branch.json
cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape branch-control-flow --receipt thir-spike-branch.json --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape checked-arithmetic --output-format json > mir-spike-checked.json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape checked-arithmetic --receipt mir-spike-checked.json --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape call-return --output-format json > mir-spike-call-return.json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape call-return --receipt mir-spike-call-return.json --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow --output-format json > mir-spike-branch.json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow --receipt mir-spike-branch.json --output-format json
```

The pilot HIR supplement is source-token-bounded review evidence; default/non-
fixture MIR and THIR contract metadata remains `unsupported-planned`. The
rustc HIR fixture spike is selected-fixture-backed/toolchain-scoped review
evidence for exactly `checked-arithmetic` (`checked.rs::checked`) and
`branch-control-flow` (`clamp.rs::clamp_to_limit`). The rustc THIR fixture
contract is selected-fixture-backed/toolchain-scoped review evidence for exactly
`checked-arithmetic` (`thir-spike/checked.rs::checked`), `call-return`
(`thir-spike/call_return.rs::call_then_return`), and `branch-control-flow`
(`thir-spike/branch_control_flow.rs::choose_branch`). THIR branch-control-flow does not imply general THIR extraction, correctness, or semantic equivalence. The rustc MIR fixture contract
is selected-fixture-backed/toolchain-scoped review evidence for exactly
`checked-arithmetic` (`mir-spike/checked.rs::checked`), `call-return`
(`mir-spike/call_return.rs::call_then_return`), and `branch-control-flow`
(`mir-spike/branch_control_flow.rs::choose_branch`). They attach to the
existing `octet.function-object.v1` `object_hash`; they do not replace it as the
review/provenance identity. Treat HIR `pilot-supported` contract metadata as
support for the checked pilot emit/check surface plus the explicit selected HIR
fixture shapes only. Treat MIR/THIR contract metadata as `unsupported-planned`
until general extractor evidence exists; only explicit selected runtime paths
(`rustc-mir-fixture --fixture-shape checked-arithmetic`, `rustc-thir-fixture
--fixture-shape checked-arithmetic`, `rustc-thir-fixture --fixture-shape
call-return`, `rustc-thir-fixture --fixture-shape branch-control-flow`, `rustc-mir-fixture --fixture-shape call-return`, or `rustc-mir-fixture --fixture-shape branch-control-flow`) may emit
`pilot-supported`. The default MIR and THIR runtime seams
remain fail-closed: non-fixture/default MIR/THIR emit/check may still replay
`unsupported-planned` records, but those records are not extractor support.
Future non-pilot HIR, THIR, or MIR families must stay
planned/unsupported until deterministic positive and negative fixture evidence
exists.

Do not describe a matching HIR/MIR/THIR supplement as a correctness proof, semantic-
equivalence proof, pure-cache eligibility proof, cross-toolchain stability
claim, or exhaustive Rust coverage claim. The check command intentionally fails
closed on schema/version, Function Object v1 hash, source path, function name,
rustc/toolchain identity, extractor identity, observation hash, and boundary
drift.

Run it whenever `cargo octet fingerprint` behavior or checked
`docs/examples/function-fingerprint-*` transcripts change. This aggregate check
is part of `nix flake check` and is the CI/operator gate for executable
fingerprint docs, including downstream smoke and the docs grep regression. Use
focused flake checks or direct Cargo filters only after the aggregate gate
identifies a failing slice.

For the internal function object corpus gate, use the named check:

```bash
nix build .#checks.x86_64-linux.object-corpus-gate --no-link -L
```

This runs the selected internal corpus fixture replay. For a freshness-only
operator rail, use the named check:

```bash
nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L
```

That focused rail compares checked-in internal, expanded, and caveat-taxonomy
object-corpus fixtures against fresh outputs. Run it before closeout whenever a
change touches object-corpus inputs such as `cargo-octet/src/object.rs`,
`cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`,
`THIR/MIR fixture inventories`, or `cargo-octet/tests/fixtures/object-corpus/*`.
Regenerate stale fixtures before relying on the aggregate gate. Treat any pass
as bounded local evidence replay for the selected corpus, not a correctness
proof, semantic-equivalence proof, cross-machine determinism proof, or exhaustive
Rust coverage claim.

For the fresh-build object corpus determinism rail, use the named check:

```bash
nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L
```

This builds two fresh `cargo-octet` derivations, regenerates the expanded
internal object corpus receipt with each binary, and compares both outputs to
the checked-in fixture. Treat a pass as bounded local fresh-build evidence
replay only; it is not a cross-machine, cross-toolchain, correctness, or
semantic-equivalence proof.

## OpenSpec touchpoints

When public docs or public runner surfaces change:

- update the active change under `openspec/changes/<change>/`
- start proposals from `openspec/templates/proposal.md`
- start delta specs from `openspec/templates/spec.md`
- start designs from `openspec/templates/design.md`
- keep `tasks.md` in the repo template shape from `openspec/templates/tasks.md`
- update `openspec/specs/documentation/spec.md` when the docs contract changes
- update `openspec/specs/consumer-integration/spec.md` when command, Nix helper, or metadata behavior changes

If a change finishes every task, sync the delta spec into `openspec/specs/` and
archive the change immediately.
