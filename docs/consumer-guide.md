# Consumer guide

Start here when onboarding Octet into a Rust workspace. Octet is the canonical
project/toolchain identity; Tiger Style names the coding philosophy, and
`slotcar` / `tigerstyle` command or config names are compatibility aliases only.
This page covers the supported runner surface, where config lives, and which
deeper docs to read next.

For the landing page and naming map, see `../README.md`.
For the full lint catalog, see `lint-reference.md`.
For public boolean API migration and narrow compatibility exceptions, see
`boolean-api-modeling.md`.
For the Rust-compatible public story cards, examples, quickstart, FAQ, and
case-study format, see `adoption-story.md`.
For rollout profiles, see `downstream-rollout.md`.
For the checked-path, evidence-stack, provenance, scope-boundary, and
controlled-deployment map, see `trust-boundary-evidence.md`.

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code: Rust source flows through Octet checks, checked-in
policy/config, reviewable evidence artifacts, and then consumer CI or release
gates.

## Supported entry points

Use the first-class Octet runner surface. Pick one of these paths:

Nix one-off:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Outside Nix, install the subcommand once, then export the bootstrap revision
used by this repo before running it:

```bash
cargo install --path /path/to/octet/cargo-octet
export OCTET_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9
cargo octet check
```

For repo-local one-offs without installation, reuse the same
`OCTET_DYLINT_REV` export and run the binary from the checkout:

```bash
cargo run --manifest-path /path/to/octet/cargo-octet/Cargo.toml -- check
```

Compiler diagnostic spans from `include_str!` and `#[path]` may contain `..`.
The runner normalizes these only for a local package matched in the complete
Cargo metadata graph (including excluded in-workspace path dependencies) and
a physically contained, existing regular source file. On the tested Linux runner,
the capability-confined open is nonblocking; the check does not prove immutable
source identity under concurrent swaps. Non-Unix platforms reject these
absolute/parent-bearing spans rather than taking a potentially blocking path.
Ordinary clean relative locators keep their previous validation; invalid or
escaped spans are errors, not silently omitted findings.
This does not weaken artifact capability paths.

For Nix consumers wiring Octet into `nix flake check`, use the first-party
helper exposed from `flake.nix`:

- `lib.mkConsumerCheck`

Historical `TIGERSTYLE_PATH` wrappers still exist in archived rollout evidence,
but they are replay-only. New consumers should start from one of the first-class
paths above plus `[workspace.metadata.octet]`.

## Quick start

### 1. Copy the consumer-owned lint config

Copy `templates/dylint.toml` into the workspace root as `dylint.toml`.

A minimal starting point can be as small as:

```toml
[octet]
```

Use the template comments when you need threshold overrides, allowlists,
`disabled_lints`, or `[octet.lint_levels]`. Legacy `[slotcar]` and
`[tigerstyle]` config remain accepted only as lower-precedence migration
aliases. Precedence is per section, not per key: any `[octet]` table, even
`[octet.lint_levels]` alone, replaces the legacy sections entirely. A shadowed
legacy section is never parsed, and the lint run warns that it is ignored.

The workspace `dylint.toml` is the only lint config source. Dylint itself would
read the `DYLINT_TOML` environment variable instead of the file, but Octet's
config hash, strict-profile scopes, and finding IDs come from the file. So
`cargo octet check` and `cargo octet admit` exit `2` before running Cargo when
`DYLINT_TOML` is set, even to an empty value. Unset it and put the
configuration in `dylint.toml`.

### 2. Add runner policy to the workspace `Cargo.toml`

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

Keep the split clear:

- `[workspace.metadata.octet]` owns runner policy
- `[octet]` in `dylint.toml` owns lint tuning
- legacy `[workspace.metadata.slotcar]` and `[slotcar]` are compatibility aliases; Octet config wins when both are present

Runner keys:

| Key | Meaning |
| --- | --- |
| `default_scope` | default `cargo check` scope when the command gets no explicit `-p` / `--workspace` flags |
| `cargo_check_args` | default cargo-check tail such as `--lib` |
| `architecture_policy` | optional workspace-relative Nickel policy pointer for project FCIS and Hexagonal checks |

Do not add `architecture_policy` until the project has complete roles, targets, exports, and freshness data. Read [`project-architecture-policy.md`](project-architecture-policy.md). Without the pointer, Octet makes no project architecture claim.

If the policy selects `admission`, `cargo octet check` runs strict lint, the architecture gate, and the explicitly bound effect-closure policy in that order.

### 3. Run the check

Run one of the supported entry points above from the workspace root. If you
installed `cargo-octet`, the common shape is:

```bash
cargo octet check
```

You can override scope or cargo-check args per run:

```bash
cargo octet check -p crate-a
cargo octet check -p crate-a -- --all-targets
```

With `--all-targets`, Cargo compiles a library both as the library and as its
unit-test harness and reports a lint in shared code once per compilation. The
runner reports a finding once when every field except its `F<n>` index agrees,
so totals, the summary, `results.jsonl`, SARIF, and baselines count it once.
Findings that differ in message, location, severity, or any other field are all
kept.

You can also choose machine-readable output, save an artifact bundle, and verify
that saved bundle before promoting it:

```bash
cargo octet check --output-format json
cargo octet check --output-format sarif --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet --output-format json
```

Once a rollout has known debt, capture it as an explicit acceptance baseline:

```bash
cargo octet check --output-format json --write-baseline octet-baseline.json
cargo octet check --output-format json --baseline octet-baseline.json
```

Do not use a finding baseline, warning budget, or broad lint disable as
completion evidence for the strict `raw_bool_parameter` migration. Inventory
the declared owned API scope, replace each covered direct boolean with a
reviewed domain type, and recheck the migrated consumer before promotion.
Only a specific, owner-attributed compatibility contract can justify one
source-local expectation; see [`boolean-api-modeling.md`](boolean-api-modeling.md).

The baseline file records the current semantic/stable finding IDs plus enough
finding context for review. File paths, lines, and columns remain display
metadata, so simple renames do not create fresh debt. A finding ID is a BLAKE3
hash of exactly these inputs:

- the bare lint name;
- the crate name (the Cargo target name that emitted the finding);
- the diagnostic message, with workspace-root paths normalized;
- the first suggested replacement text, if any;
- the hash of the primary span's source snippet with whitespace runs collapsed,
  or a fixed `unsupported` marker when the span has no source text;
- the run's config hash;
- the `cargo-octet` version.

The config hash covers only lint-relevant configuration: the `dylint.toml`
bytes, the canonical (key-order-independent) contents of the
`[workspace.metadata.octet]` table and the legacy `[workspace.metadata.slotcar]` /
`[workspace.metadata.tigerstyle]` tables in the root `Cargo.toml`, the effective
scope arguments, the effective `cargo check` arguments, and the strict profile
identity (profile ID and policy hash) when a profile is selected. "Effective"
means after defaults apply: running with no scope flags uses `default_scope`, so
it matches an explicit run with the same flags, but `--workspace` and
`-p mycrate` are different scopes, and adding or reordering `-- <cargo check
args>` changes the arguments. Other manifest edits, such as dependency bumps,
new workspace members, or `[package]` changes, keep finding IDs stable. A root
`Cargo.toml` that cannot be parsed falls back to hashing its raw bytes.

Write and check a baseline with the same scope, the same `cargo check`
arguments, the same `dylint.toml` and runner metadata, the same strict profile,
and the same `cargo-octet` version. Changing any of them re-keys every finding:
all current findings report as new and every accepted ID reports as stale.
Regenerate the baseline with `--write-baseline` after such a change.

Releases before the manifest-table rule hashed the whole root `Cargo.toml`, so
upgrading re-keys existing baselines once; regenerate them with
`--write-baseline` after upgrading. Future `--baseline` runs compare the current
finding set to that accepted ID set. Fully accepted warning-only findings keep
exit `0`; any new unaccepted stable ID fails the run with exit `1` and records
`baseline` counts in `status.json` / SARIF properties. Stale accepted IDs are
reported so teams can prune resolved debt without blocking the gate.
Stale IDs need a complete scan, the same one `--write-baseline` requires. When
`cargo check` did not succeed, it may have stopped before checking a crate whose
accepted IDs are therefore missing but not resolved, so the evaluation records
`scan_complete: false`, sets `stale_total` and `stale_finding_ids` to `null`,
and the summary reports the stale count as unknown. New IDs are still counted
and still gate the run.

Exit codes are stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration/configuration failures, including underlying `cargo check` failures

rustc fails a compile that hit a denied lint, so `cargo check` itself exits
non-zero for error-level Octet findings. The runner still reports exit `1`
when Cargo exited with a code and every compiler error is an Octet finding;
any other compiler error (an `E####` code, an uncoded error, or an internal
compiler error) or a Cargo process ended by a signal stays exit `2`.
`--write-baseline` records only a complete scan, one where `cargo check`
itself succeeded. Denied lints make Cargo fail and stop before every crate is
checked, so such a run writes no baseline and exits `2` with an error naming
the cause; lower those lints to `warn` for the baseline run. Lower them outside
`dylint.toml`, for example by running without the deny-all hook or with
`-W <lint>` in `DYLINT_RUSTFLAGS`: the `dylint.toml` bytes are part of every
finding ID, so a baseline written under an edited `dylint.toml` would not match
later runs. A baseline cannot make error-level findings pass in any case.

SARIF invocations record the same `exitCode`. `executionSuccessful` is `false`
only for exit `2`; a complete scan that fails on error-level findings or new
baseline IDs (exit `1`) is still a successful tool execution.

The SARIF run declares `"columnKind": "unicodeCodePoints"`, because columns come
from rustc, which counts Unicode scalar values. Each `artifactLocation.uri` is
the workspace-relative path with every byte outside RFC 3986 path characters
percent-encoded, as well as `%` and `:`, so `src/my module.rs` becomes
`src/my%20module.rs`. JSON Lines findings keep the raw path in `file`.

When `--artifact-dir` is set, the runner writes `command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`, and the selected structured-output mirror (`results.jsonl` or `results.sarif`). The directory must be new, empty, or an existing `cargo octet check` bundle, recognized by both `status.json` and `provenance.jsonl` being present as regular files and by a `status.json` that declares no `bundle_kind` or `schema_version` (prove and admission bundles declare one); only then does the runner prune leftover check outputs (for example a stale `results.sarif` from an earlier SARIF run) before writing. Pruning removes only names a check bundle can hold (the sidecars above, both structured mirrors, `function-addresses.json`, and the `compiler-architecture-*`, `project-architecture-*`, and `effect-closure*` artifacts); any other file or directory in the bundle is left in place, and `cargo octet artifact verify` reports it as an unexpected artifact. Any other non-empty directory, such as Cargo's `target/` or a prove or admission bundle, is left untouched and the run fails as an integration failure (exit `2`). JSON findings carry `stable_id`, `semantic_id`, `identity_status`, `normalized_snippet_hash`, `effect_family`, `evidence_tier`, and `cache_status`; `provenance.jsonl` records source IDs, checker IDs, evidence tiers, artifact locators, BLAKE3 content hashes, and checked scopes for the saved sidecars. `cargo octet artifact verify --artifact-dir artifacts/octet` is non-mutating and fails closed when the bundle is partial, stale, malformed, hash-mismatched, uses an unknown checker ID, uses an invalid evidence tier, or lacks required manual-review action text. A bundle is partial when `status.json` declares `phases.architecture` or a non-null `phases.effect_closure` but the matching `project-architecture-*` or `effect-closure*.json` artifacts are absent, even if their provenance lines were removed too. When `project-architecture-receipt.json` is present, `status.json` `phases.effect_closure` must equal the receipt's effect-closure link (`null` when it has none). `status.json`, SARIF run properties, baseline evaluations, and stderr summaries include replay metadata such as the toolchain plus config/profile hashes. Fingerprint review manifests also emit a sibling `.provenance.jsonl` sidecar, and transcript JSON output carries an `octet-transcript` provenance record. Downstream tools can compare runs without scraping ambient stderr.

The artifact directory holds the check bundle alone. `--artifact-dir` must not be the workspace root or one of its ancestors, and a `--baseline` or `--write-baseline` path or the `--cache` directory (`.octet/cache` under the workspace root) must not lie inside it. The runner resolves symlinks and `..` components and refuses such a combination with exit `2` before Cargo runs.

`--cache` labels findings; it does not reuse results. Every run still lints
every selected crate and reports its own findings. The runner keeps
`.octet/cache/findings.json` as one entry per key, each holding the semantic IDs
of the latest complete `--cache` run with that key. The key is the
`cargo-octet` version, the toolchain name and `rustc -Vv` version line, the
config hash (which covers the effective scope and `cargo check` arguments), and
the crate graph hash (the root `Cargo.toml` and `Cargo.lock` bytes). A finding
reports `cache_status: hit` when that entry holds its semantic ID, and `miss`
otherwise. A run replaces only its own key's entry, so `cargo octet check
--cache -p a` leaves the entry of a full-workspace run alone; the file keeps the
eight most recently written keys. A run whose `cargo check` did not succeed may
have skipped crates, so it is labelled but records nothing. Findings without a
computed identity report `bypassed`, and so does every finding when the runner
cannot read the rustc version; such a run also records nothing. Each run writes
the file to its own hidden `.findings.json.<pid>.<n>.octet-stage` file and
renames it into place, so concurrent `--cache` runs in one workspace (a
pre-commit hook and an editor, say) both succeed and the last one to finish
wins. A run killed mid-write can leave its hidden stage file behind; it is
never read and can be deleted. A cache file the runner cannot parse, including an
empty or truncated one or one from a release before the key existed, counts as
empty and is replaced by the next complete run. Without `--cache`, findings
report `disabled`.

### 3a. Optional formal proof rails

Formal proof rails are separate from the starter lint path. Consumers can list
the selected rails without a verifier installation; `--source` is required in
every mode:

```bash
cargo octet prove --list --policy-artifacts-dir artifacts/policy --source src/verified
```

`--explain` does not inspect: it runs the selected rails, so it needs the
verifiers, and prefixes each failure diagnostic with remediation context:

```bash
cargo octet prove --explain --policy-artifacts-dir artifacts/policy --source src/verified \
  --rail octet.verus_proof_rails.addition-bound-production#rail-0
```

A proof run executes only configured rail `command_argv` entries, captures
bounded stdout/stderr artifacts, writes a structured JSON receipt per rail, and
records `formal_proof` provenance only when the checked source/proof paths and
receipt paths match the rail scope. Fixture rails must say
`verifier_mode = "fixture"`; production Verus rails must say
`verifier_mode = "production"` and pin verifier, solver, toolchain, Nix closure,
command, source/proof, receipt, and trust-boundary identity in the receipt.
Before running a production rail, `cargo octet prove` asks the resolved verifier
for its `--identity` report and fails the rail with an `unpinned-verifier`
diagnostic unless the reported verifier binary, solver, and toolchain BLAKE3
digests equal the rail's pins. The probe covers every production rail except an
`aeneas-lean` rail, whatever its `backend` label, because `cargo octet prove`
issues a structured Verus receipt for each of them.
Octet's built-in production pilots are intentionally tiny and run
`octet-production-verus` only when selected proof rails match the requested
source scope. Each pilot has its own source path, proof path, receipt path,
checked scope, and property-specific non-claim wording. The repository also has
an opt-in invalid production fixture that runs only from its failure-fixture
source selector; it demonstrates non-zero Verus rejection, `verifier-failure`
diagnostics, bounded receipt/log identities, and no `formal_proof` provenance.

```bash
cargo octet prove --policy-artifacts-dir artifacts/policy --source src/verified --artifact-dir artifacts/octet-proofs
cargo octet artifact verify --artifact-dir artifacts/octet-proofs
```

`cargo octet check` and `cargo octet prove` write separate bundles, so give each
its own artifact directory; `cargo octet prove` refuses a directory that holds a
`check` bundle. Reusing an earlier prove bundle removes only the entries prove
writes (`command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`,
`proof-cards.json`, `evidence-log-manifest.json`, and the `proofs/` tree), so a
`proof-ref.json` or other file kept beside them survives; `cargo octet artifact
verify` reports it as an unexpected artifact. `--artifact-dir` must not be the
proof sources' workspace root or one of its ancestors; prove refuses it with
exit `2` before any rail runs.

External Aeneas consumers use a closed Nickel profile. The receipt binds the
provider cohort, exact source and span, generated artifacts, theorem,
requirement, trust closure, resource policy, and outcome. Use
[`aeneas-consumer-contract.md`](aeneas-consumer-contract.md) for the checked
fixture, migration rules, and fail-closed cases.

Starter `cargo octet check` usage does not require Verus.

### 3b. Optional external evidence rails

External tool rails are opt-in evidence, not starter checks. `cargo octet evidence collect` imports saved tool output or runs a selected tool command, then writes `evidence-rail-receipt.json`, `summary.txt`, and `provenance.jsonl` under the chosen artifact directory. The `--artifact-dir` directory must be new, empty, or an existing evidence bundle, recognized by both `evidence-rail-receipt.json` and `provenance.jsonl` being present as regular files; only then does the collector prune leftover rail outputs from the earlier bundle before writing. Pruning removes only names an evidence rail writes (the three sidecars, rail payloads such as `sarif-raw-report.json` or `coverage-comparison.json`, and `mutation-log-*`/`mutation-diff-*` logs); any other entry is left in place, and `cargo octet artifact verify` reports it as an unexpected artifact. Any other non-empty directory, such as Cargo's `target/`, is left untouched and the command exits `2` without writing a receipt. `--artifact-dir` must not be the workspace root (`--workspace-root`, or the current directory when it is omitted) or one of its ancestors; such a directory is refused with exit `2` before any tool runs.

```bash
cargo octet evidence collect --rail dependency-audit --input target/deny.json --config deny.toml --locked --artifact-dir artifacts/dependency-audit
cargo octet evidence collect --rail clippy-co-reporting --input target/clippy.jsonl --artifact-dir artifacts/clippy
cargo octet evidence collect --rail clippy-co-reporting --run --profile production-safety --workspace-root . --artifact-dir artifacts/clippy-restrictions
cargo octet evidence collect --rail dependency-trust --input target/vet.json --artifact-dir artifacts/dependency-trust
cargo octet evidence collect --rail api-stability --input target/semver.json --baseline baselines/public-api --artifact-dir artifacts/api-stability
cargo octet evidence collect --rail dynamic-evidence --input target/miri-profile.json --profile miri-unsafe-boundary --artifact-dir artifacts/dynamic/miri
cargo octet evidence collect --rail dynamic-evidence --input target/cargo-fuzz-run.json --profile cargo-fuzz-libfuzzer --config dynamic-evidence-profiles.json --artifact-dir artifacts/dynamic/cargo-fuzz
cargo octet evidence collect --rail sarif-import --input target/codeql.sarif --profile codeql-local --config standards/sarif/profiles.json --artifact-dir artifacts/sarif-codeql
cargo octet evidence collect --rail release-supply-chain --input target/release-supply-chain.json --parent-artifact artifacts/octet/status.json --artifact-dir artifacts/release-supply-chain
cargo octet evidence collect --rail mutation-testing --profile cargo-mutants-octet-metadata --config standards/mutation/profiles.json --input .octet/mutants-output/mutants.out/outcomes.json --external-report evidence/mutation-source-manifest.json --workspace-root . --artifact-dir artifacts/mutation-testing
cargo octet evidence collect --rail coverage-baseline --profile cargo-llvm-cov-octet-lib --config standards/coverage/profiles.json --input target/coverage.json --baseline baselines/coverage.json --external-report evidence/coverage-source-manifest.json --workspace-root . --artifact-dir artifacts/coverage-baseline
cargo octet evidence collect --rail benchmark-evidence --profile octet-metadata-benchmark --config standards/benchmark/profiles.json --input evidence/benchmark-input.json --workspace-root . --artifact-dir artifacts/benchmark-evidence
cargo octet evidence collect --rail kani-model-check --input target/kani-summary.json --artifact-dir artifacts/kani-model-check
cargo octet evidence collect --rail hegel-property --profile octet-pure-core-laws-v1 --config standards/hegel/profiles.json --workspace-root . --run --locked --artifact-dir artifacts/hegel-property
cargo octet evidence collect --rail hegel-property --input target/hegel-property-v1.json --artifact-dir artifacts/hegel-property-import
cargo octet evidence collect --rail binary-debug-map --input target/release/example --debug-companion target/release/example.debug --config standards/binary-debug-map/profiles.json --profile elf-dwarf-v1 --function-address-artifact artifacts/function-addresses.json --artifact-dir artifacts/binary-debug-map
cargo octet artifact verify --artifact-dir artifacts/dependency-audit
```

Relative `--config` and `--baseline` paths resolve against `--workspace-root` (default `.`), the directory where run mode starts the selected tool and where `Cargo.lock` is read, so the receipt identity always binds the same bytes the tool or profile check read. The `ast-grep`, `wasm-artifact`, `charon-semantic`, and profiled `clippy-co-reporting` rails, which read and verify their own `--config` profile, keep resolving it against the current directory. The `sarif-import` receipt binds the checked SARIF profile identity after verifying the `--config` bytes under the workspace root.

Run mode executes the selected tool through Bounded Exec in its own process group, with the ambient environment. Each of stdout and stderr keeps at most 16 MiB, and a run may last four hours. If the tool writes more than 16 MiB to either stream, outlives the deadline, or is killed by a signal, Octet exits `2` without a receipt; Octet never parses a truncated transcript. The profiled cargo-fuzz, coverage, mutation, benchmark, and Hegel producers keep the limits of their own profiles instead.

The `dependency-audit` input must be a `cargo-deny-diagnostics/v1` document with a `diagnostics` array; a document without one fails. Run mode executes `cargo deny --format json --config PATH check`, adding `--locked --offline` before `check` when `--locked` is given without `--update-advisory-database`. The `--offline` flag stops cargo-deny from fetching advisory databases, so the receipt's `command_argv` shows the flag and `advisory_database_update` records `reused`. Refresh advisories with `cargo deny fetch` as a separate earlier step. Octet converts the JSON lines that cargo-deny writes to stderr into a `cargo-deny-diagnostics/v1` document. Each diagnostic keeps its cargo-deny `code`. Its category is the check that owns the code in cargo-deny 0.20.2 (`advisories`, `bans`, `licenses`, or `sources`), except that duplicate-version codes become `duplicates` and codes without an owning check, such as the general `deprecated` code, a diagnostic without a code, or a code from a newer cargo-deny, become `unclassified`. Error and warning diagnostics with a known code must match their check's counts in cargo-deny's summary line. cargo-deny does not count its configuration-validation diagnostics, such as the `deprecated` warning for a moved deny.toml key or the code-less warning for an unknown target triple, so those stay `unclassified` findings without being reconciled. Every other `unclassified` diagnostic must account for exactly the errors and warnings that the summary reports beyond the known codes, summed across checks. An `unclassified` error fails the receipt like any other error. The run fails without a receipt when cargo-deny's output has a non-JSON or unknown record, has no summary line or more than one, or reports error or warning counts in its summary that the diagnostics it emitted do not reconcile.

The `dependency-trust` rail reads `cargo vet --output-format=json` output. A `success` conclusion passes when every package is listed in `vetted_fully`. Each package in `vetted_partially` or `vetted_with_exemptions` becomes an `exemption` finding that requires manual review. Each `failures` entry of a `fail (vetting)` conclusion, and each `violations` key of a `fail (violation)` conclusion, becomes an `unaudited-gap` error. A document without a supported `conclusion`, including cargo-vet's `{"error": ...}` report, fails. cargo-vet's JSON does not name the audit imports it relied on, so in both `--run` and `--input` modes Octet reads the cargo-vet store under `--workspace-root`: `supply-chain/`, or the `store.path` set in `[package.metadata.vet]` or `[workspace.metadata.vet]` of the workspace `Cargo.toml`. The receipt's `dependency_trust` object records the BLAKE3 identity of the store's `config.toml`, the name and `url` of each `[imports.NAME]` entry it declares, and the BLAKE3 identity of the store's `imports.lock`. The receipt fails when the store `config.toml` is missing or malformed, when an import declares no `url`, or when the config declares imports but `imports.lock` is absent. Receipt validation likewise rejects a `dependency-trust` receipt without the `dependency_trust` object, or one whose declared imports have no `imports.lock` identity.

The `api-stability` rail passes `--baseline` to `cargo semver-checks check-release --baseline-root`, and the receipt `command_argv` records that baseline path. A baseline crate directory gets a deterministic tree identity: name-sorted entries, file contents, symlink targets, and subdirectories are hashed recursively, while `target/` and `.git/` are skipped as build and VCS state.

Read [`mutation-testing-evidence.md`](mutation-testing-evidence.md) for typed cargo-mutants profiles, complete outcome partitions, and retained diffs.

Read [`coverage-baseline-evidence.md`](coverage-baseline-evidence.md) for the checked coverage profile, exact-count comparison, and baseline-envelope workflow.

Read [`benchmark-evidence.md`](benchmark-evidence.md) for typed workloads, bounded samples, resource observations, signal oracles, comparisons, and public-claim boundaries.

Read [`mirai-semantic-pilot.md`](mirai-semantic-pilot.md) for the bounded blocked pilot. The pilot is not a production evidence rail.

The dependency-audit rail uses tool-native `deny.toml`; `templates/deny.toml` is a starter copy. Clippy restriction run policy is Nickel-authored at `standards/clippy-restrictions/profiles.ncl` and embedded in `cargo-octet` for downstream selection by profile name; `--config PATH` may supply another checked catalog. Profiled runs bind exact catalog/profile BLAKE3 identities and fixed lint argv, reject arbitrary `--command` authority, and never enable the blanket `clippy::restriction` group. Use `production-safety` for general production code, `canonical-determinism` only where byte/order contracts apply, and `shell-boundary-inventory` as a warning-only cleanup rail. Dynamic evidence profile configuration is Nickel-authored at `templates/dynamic-evidence-profiles.ncl`. The cargo-fuzz profile rejects arbitrary commands and binds one exact target, seed, budget, corpus root, crash evidence set, and transcript tier. The Kani rail imports a Rust-owned summary for named harnesses, toolchain/solver identity, unwind or contract status, and stub classification. The Hegel rail imports a Rust-owned summary for named pure-core property IDs, linked requirement IDs, source scope, runner settings, generator identity, BLAKE3 source hashes, and counterexample identity; `--run` uses the fixed checked profile and rejects arbitrary command authority. These receipts preserve checker identity (`cargo-deny`, Clippy, `cargo-vet`, `cargo-semver-checks`, `cargo-mutants`, `cargo-llvm-cov`, `octet-benchmark`, `cargo-kani`, `hegel-rs`, dynamic profile runner, or release supply-chain collector), source severity, selected scope, BLAKE3 input identity, parent artifacts when supplied, and explicit non-claims. Clippy co-reporting remains external `static_lint` evidence with checker ID `clippy`; mutation, coverage, benchmark, dynamic, Kani, and Hegel rails remain `transcript_assertion` evidence; the other rails remain `policy_check` evidence. They do not replace Octet source lints, cargo-vet human review, legal review, vulnerability triage, test review, uncovered-critical-path review, formal proof, Kani verifier-soundness review, or Hegel generated-testing boundary review.

### 3c. Controlled deployment quick check

Octet's controlled-deployment posture is local-first and consumer-owned:

- local or consumer-CI execution through `cargo octet check`, `nix run ...#cargo-octet -- check`, or `lib.mkConsumerCheck`;
- no required hosted service or Octet-operated upload path;
- pin-capable runner path using a reviewed release tag, commit, or local `path:` flake reference;
- source-visible toolchain in this repository: runner, lint library, standards checker, policy registry, and generated artifact seam;
- local artifact directory with `cargo octet check --artifact-dir artifacts/octet`, followed by `cargo octet artifact verify --artifact-dir artifacts/octet` before release promotion.

Octet itself does not require uploading source or artifacts. Consumers still own
what their CI storage, logs, artifact retention, and integrations do after Octet
writes local files. Regulated consumers should pin the Octet revision, check in
`dylint.toml` and `[workspace.metadata.octet]`, choose a deterministic artifact
directory, and optionally add the deny-all pre-commit gate below.

### 3d. Add the deny-all pre-commit hook

Copy `templates/pre-commit-config.yaml` to the consumer workspace as
`.pre-commit-config.yaml`, pin `rev` to a reviewed Octet release tag or commit,
and install pre-commit normally:

```bash
pre-commit install
pre-commit run octet-deny-all --all-files
```

The hook is published from this repository through `.pre-commit-hooks.yaml`.
It runs once from the consumer repository root, ignores pre-commit's per-file
argument list, and delegates to `cargo octet check` using the consumer's
`[workspace.metadata.octet]` scope defaults. By default it uses Nix to run the
same pinned hook checkout:

```yaml
repos:
  - repo: https://github.com/OnixResearch/octet
    # Replace with a reviewed release tag or commit SHA.
    rev: 0123456789abcdef0123456789abcdef01234567
    hooks:
      - id: octet-deny-all
```

The deny-all behavior comes from `DYLINT_RUSTFLAGS`: the hook appends
`-D unknown_lints` plus `-D <lint-name>` for every current Octet/Tiger Style
lint. This activates allow-by-default Octet lints and promotes warning lints to
hard errors without promoting unrelated rustc warnings. Explicit command-line
lint levels win over `[octet.lint_levels]`, so a consumer `numeric_units =
"warn"` entry cannot weaken the hook's `-D numeric_units`. Under the hook,
exact nominal-domain findings are denied, while suffix-only
`primitive_domain_alias` discovery candidates are skipped rather than failing
the build. Do not use `disabled_lints` with this strict hook; disabled Octet
lints are intentionally fail-closed because their command-line deny name is no
longer registered.
Prefer narrow source `allow` attributes with reasons for reviewable exceptions.

If a consumer cannot use Nix but already installed `cargo-octet`, set
`OCTET_PRECOMMIT_USE_INSTALLED=true` in that environment. Use hook `args` only
for runner scope or cargo-check tail overrides, for example:

```yaml
      - id: octet-deny-all
        args: ["--workspace", "--", "--all-targets"]
```

### 3e. Plan structured migrations

Use the migration manifest commands when rolling out a new profile or policy set:

```bash
cargo octet migrate start --profile starter --manifest .octet/migrations/starter.json
cargo octet migrate status --manifest .octet/migrations/starter.json
cargo octet migrate apply-safe --manifest .octet/migrations/starter.json
```

`start` writes a deterministic manifest with config and crate-graph hashes.
`status` reports accepted/fixed/deferred/manual/unresolved counts. `apply-safe`
only records safe-apply state and preserves manual-review buckets; code edits
still flow through the existing machine-applicable cargo-fix rails.
Each subcommand accepts only its own flags: `--profile` is `start`-only,
`--manifest` belongs to `start`/`status`/`apply-safe`, `--session` to
`migrate session`, and `--operation-manifest`/`--dry-run` to
`migrate session apply`. SARIF output is rejected too; rejected invocations
exit 2 before any manifest is read or written.

### 3f. Check documentation transcript structure

Markdown examples can carry fenced transcript blocks with `$ ` command lines and
`=> ` expected-output lines:

```octet-transcript
$ cargo octet check --output-format json
=> {"version":1
```

Check a document's structure with:

```bash
cargo octet transcript docs/examples/my-transcript.md --format json
```

`cargo octet transcript` is a structural check only. It confirms that every
fence holding `$ ` lines uses `octet-transcript`, `slotcar-transcript`,
`console`, or `sh`, that no fence is left open, and that the document has at
least one `$ ` command line and one `=> ` expected-output line. It does not run
the commands or compare their output with the `=> ` lines. The report records
the document's BLAKE3 as `transcript_hash`, so a reviewer can pin the exact
transcript bytes; behavior claims still need a gate that runs the commands.

### 4. Tune rollout controls in `dylint.toml`

Keep config keys bare in `dylint.toml`:

```toml
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
numeric_units = "warn"
```

Explicit command-line lint levels, such as `-D numeric_units` in
`DYLINT_RUSTFLAGS`, win over `[octet.lint_levels]`. A selected strict
profile's required denies win over both.

Source attrs and group attrs stay canonical in Rust code:

```rust
#[allow(tigerstyle::unchecked_narrowing)]
#![deny(tigerstyle::naming)]
```

## Nix integration

The flake publishes the same runner surface through `nix run`,
`lib.mkConsumerCheck`, and `lib.mkPolicyArtifactConsumerCheck`.

Use `nix run` for one-off checks:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Use `lib.mkConsumerCheck` when you want Octet inside downstream flake
checks.

Use `lib.mkPolicyArtifactConsumerCheck` when a downstream flake should consume
the generated policy artifact bundle rather than reading the Nickel registry at
check time:

```nix
checks.${system}.octet-standards = octet.lib.mkPolicyArtifactConsumerCheck {
  inherit system;
  src = self;
  family = "embedded";
  sourceSubdirs = [
    "crates/my-core"
    "crates/my-shell"
  ];
  expectedDiagnostics = 0;
  requiredPolicyIds = [ "tigerstyle.embedded.bounded-queues" ];
};
```

The helper exports and asserts the generated bundle files, copies the consumer
source into the check sandbox, runs `octet-standards check
--policy-artifacts-dir ...`, asserts the expected diagnostic count, and can
assert required policy IDs are present in the policy-evaluation metadata. With
`expectedDiagnostics = 0` the checker must exit 0; with a nonzero count, its
blocking-findings exit is accepted only when the JSON report carries blocking
diagnostics, so configuration or usage errors still fail the check, and an
exit 0 whose report carries a blocking diagnostic fails too. For
`family = "embedded"`, the generated artifact drives the
`tigerstyle.embedded.bounded-queues` structural gate: forbidden unbounded
constructors fail, while approved bounded APIs or wrappers need explicit
capacity/backpressure witnesses. Legacy `slotcar.*` policy IDs are compatibility
aliases only when a specific policy contract says so.

Wrapped Nix outputs already provide the runtime environment expected by the
runner, including:

- `OCTET_DRIVER_PATH`
- `OCTET_LINT_LIB`
- `OCTET_TOOLCHAIN`

Replay or bootstrap helpers may also set `OCTET_DYLINT_REV`.

## Runtime environment contract

The public runner surface uses the `OCTET_*` namespace for support wiring. Most
consumers should prefer the wrapped Nix app or repo-managed automation instead
of exporting these variables by hand.

Current canonical names that appear in wrappers and support tooling:

- `OCTET_DRIVER_PATH`
- `OCTET_LINT_LIB`
- `OCTET_TOOLCHAIN`
- `OCTET_DYLINT_REV` for driver bootstrap and replay helpers

The previous `TIGERSTYLE_*` variables (`TIGERSTYLE_DRIVER_PATH`,
`TIGERSTYLE_LINT_LIB`, `TIGERSTYLE_TOOLCHAIN`, `TIGERSTYLE_DYLINT_REV`) are
compatibility aliases while Dylint bootstrap wiring migrates. The runner reads
each `OCTET_*` name first and uses the alias only when the `OCTET_*` name is
unset. An empty or non-UTF-8 value in the variable the runner reads is an
error; it does not fall back to the alias. `TIGERSTYLE_PATH` belongs to
historical wrapper-based replay flows, not the current recommended onboarding
path.

## Migration note: `todo!()` escapes moved to `no_todo`

If a workspace still uses justified `todo!()` stubs under `allow(no_panic)` or
`allow(tigerstyle::no_panic)`, migrate them to `no_todo`:

- `#[allow(no_todo)]`
- `#[allow(tigerstyle::no_todo)]`

`no_panic` now covers only:

- `panic!()`
- `unimplemented!()`
- `unreachable!()`

Use `unjustified_no_todo_allow` to keep stub escapes documented and local.

## Next docs

- `lint-reference.md` for the full lint catalog, defaults, and config keys
- `adoption-story.md` for the Rust-compatible public story cards, examples, quickstart, FAQ, case-study fields, and methodology boundaries
- `downstream-rollout.md` for smoke, boundary-first, and inventory-only rollout profiles
- `downstream-rollout-evidence.md` for Aspen-backed evidence and historical replay paths
- `contributor-guide.md` if you are changing lints, config keys, or published runner surfaces
