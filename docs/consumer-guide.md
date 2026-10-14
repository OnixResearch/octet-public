# Consumer guide

Start here when onboarding Octet into a Rust workspace. Octet is the canonical
project/toolchain identity; Tiger Style names the coding philosophy, and
`slotcar` / `tigerstyle` command or config names are compatibility aliases only.
This page covers the supported runner surface, where config lives, and which
deeper docs to read next.

For the landing page and naming map, see `../README.md`.
For the full lint catalog, see `lint-reference.md`.
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
`disabled_lints`, or `[octet.lint_levels]`. Legacy `[slotcar]` config remains
accepted only as a lower-precedence migration alias.

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

The baseline file records the current semantic/stable finding IDs plus enough
finding context for review. Semantic IDs are derived from the lint, crate,
message/suggestion, normalized snippet hash when available, config hash, and
Octet tool version; file paths remain display metadata so simple renames do not
create fresh debt. Future `--baseline` runs compare the current finding set to
that accepted ID set. Fully accepted warning-only findings keep exit `0`; any new
unaccepted stable ID fails the run with exit `1` and records `baseline` counts in
`status.json` / SARIF properties. Stale accepted IDs are reported so teams can
prune resolved debt without blocking the gate.

Exit codes are stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration/configuration failures, including underlying `cargo check` failures

When `--artifact-dir` is set, the runner writes `command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`, and the selected structured-output mirror (`results.jsonl` or `results.sarif`). JSON findings carry `stable_id`, `semantic_id`, `identity_status`, `normalized_snippet_hash`, `effect_family`, `evidence_tier`, and `cache_status`; `provenance.jsonl` records source IDs, checker IDs, evidence tiers, artifact locators, BLAKE3 content hashes, and checked scopes for the saved sidecars. `cargo octet artifact verify --artifact-dir artifacts/octet` is non-mutating and fails closed when the bundle is partial, stale, malformed, hash-mismatched, uses an unknown checker ID, uses an invalid evidence tier, or lacks required manual-review action text. `status.json`, SARIF run properties, baseline evaluations, and stderr summaries include replay metadata such as the toolchain plus config/profile hashes. Fingerprint review manifests also emit a sibling `.provenance.jsonl` sidecar, and transcript JSON output carries an `octet-transcript` provenance record. Downstream tools can compare runs without scraping ambient stderr.

### 3a. Optional formal proof rails

Formal proof rails are separate from the starter lint path. Consumers can inspect
available rails without a verifier installation:

```bash
cargo octet prove --list --policy-artifacts-dir artifacts/policy
cargo octet prove --explain --policy-artifacts-dir artifacts/policy
```

A proof run executes only configured rail `command_argv` entries, captures
bounded stdout/stderr artifacts, writes a structured JSON receipt per rail, and
records `formal_proof` provenance only when the checked source/proof paths and
receipt paths match the rail scope. Fixture rails must say
`verifier_mode = "fixture"`; production Verus rails must say
`verifier_mode = "production"` and pin verifier, solver, toolchain, Nix closure,
command, source/proof, receipt, and trust-boundary identity in the receipt.
Octet's built-in production pilots are intentionally tiny and run
`octet-production-verus` only when selected proof rails match the requested
source scope. Each pilot has its own source path, proof path, receipt path,
checked scope, and property-specific non-claim wording. The repository also has
an opt-in invalid production fixture that runs only from its failure-fixture
source selector; it demonstrates non-zero Verus rejection, `verifier-failure`
diagnostics, bounded receipt/log identities, and no `formal_proof` provenance.

```bash
cargo octet prove --policy-artifacts-dir artifacts/policy --source src/verified --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

Starter `cargo octet check` usage does not require Verus.

### 3b. Optional external evidence rails

External tool rails are opt-in evidence, not starter checks. `cargo octet evidence collect` imports saved tool output or runs a selected tool command, then writes `evidence-rail-receipt.json`, `summary.txt`, and `provenance.jsonl` under the chosen artifact directory.

```bash
cargo octet evidence collect --rail dependency-audit --input target/deny.json --config deny.toml --locked --artifact-dir artifacts/dependency-audit
cargo octet evidence collect --rail clippy-co-reporting --input target/clippy.jsonl --artifact-dir artifacts/clippy
cargo octet evidence collect --rail dependency-trust --input target/vet.json --artifact-dir artifacts/dependency-trust
cargo octet evidence collect --rail api-stability --input target/semver.json --baseline baselines/public-api --artifact-dir artifacts/api-stability
cargo octet evidence collect --rail dynamic-evidence --input target/miri-profile.json --profile miri-unsafe-boundary --artifact-dir artifacts/dynamic/miri
cargo octet evidence collect --rail release-supply-chain --input target/release-supply-chain.json --parent-artifact artifacts/octet/status.json --artifact-dir artifacts/release-supply-chain
cargo octet evidence collect --rail mutation-testing --input target/mutants.json --artifact-dir artifacts/mutation-testing
cargo octet evidence collect --rail coverage-baseline --input target/coverage.json --baseline baselines/coverage.json --artifact-dir artifacts/coverage-baseline
cargo octet evidence collect --rail kani-model-check --input target/kani-summary.json --artifact-dir artifacts/kani-model-check
cargo octet evidence collect --rail hegel-property --input target/hegel-property.json --artifact-dir artifacts/hegel-property
cargo octet artifact verify --artifact-dir artifacts/dependency-audit
```

The dependency-audit rail uses tool-native `deny.toml`; `templates/deny.toml` is a starter copy. Dynamic evidence profile configuration is Nickel-authored at `templates/dynamic-evidence-profiles.ncl`. The Kani rail imports a Rust-owned summary for named harnesses, toolchain/solver identity, unwind or contract status, and stub classification. The Hegel rail imports a Rust-owned summary for named pure-core property IDs, linked requirement IDs, source scope, runner settings, generator identity, BLAKE3 source hashes, and counterexample identity; `--run` requires an explicit `--command` and never bootstraps Hegel automatically. These receipts preserve checker identity (`cargo-deny`, Clippy, `cargo-vet`, `cargo-semver-checks`, `cargo-mutants`, `cargo-llvm-cov`, `cargo-kani`, `hegel-rs`, dynamic profile runner, or release supply-chain collector), source severity, selected scope, BLAKE3 input identity, parent artifacts when supplied, and explicit non-claims. Clippy co-reporting remains external `static_lint` evidence with checker ID `clippy`; mutation, coverage, dynamic, Kani, and Hegel rails remain `transcript_assertion` evidence; the other rails remain `policy_check` evidence. They do not replace Octet source lints, cargo-vet human review, legal review, vulnerability triage, test review, uncovered-critical-path review, formal proof, Kani verifier-soundness review, or Hegel generated-testing boundary review.

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
hard errors without promoting unrelated rustc warnings. Do not use
`disabled_lints` with this strict hook; disabled Octet lints are intentionally
fail-closed because their command-line deny name is no longer registered.
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

### 3f. Verify executable documentation transcripts

Markdown examples can carry fenced transcript blocks with `$ ` command lines and
`=> ` expected-output assertions:

```octet-transcript
$ cargo octet check --output-format json
=> {"version":1
```

Verify a document with:

```bash
cargo octet transcript docs/examples/my-transcript.md --format json
```

The transcript report is content-addressed with `transcript_hash` so docs can be
stored as deterministic regression evidence.

### 4. Tune rollout controls in `dylint.toml`

Keep config keys bare in `dylint.toml`:

```toml
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
numeric_units = "warn"
```

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
  requiredPolicyIds = [ "octet.embedded.bounded-queues" ];
};
```

The helper exports and asserts the generated bundle files, copies the consumer
source into the check sandbox, runs `octet-standards check
--policy-artifacts-dir ...`, asserts the expected diagnostic count, and can
assert required policy IDs are present in the policy-evaluation metadata. For
`family = "embedded"`, the generated artifact drives the
`octet.embedded.bounded-queues` structural gate: forbidden unbounded
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

The previous `TIGERSTYLE_*` variables are compatibility aliases while Dylint
bootstrap wiring migrates. `TIGERSTYLE_PATH` belongs to historical wrapper-based
replay flows, not the current recommended onboarding path.

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
