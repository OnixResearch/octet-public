## Context

`tigerstyle` already has the right high-level contract on paper: warning-only
runs should stay non-fatal, lint failures should be distinct from integration
failures, and structured output should stay valid even on error paths. The
actual runner does not yet uphold that contract.

`cargo-tigerstyle/src/engine.rs` still returns `1` whenever the child cargo
status is non-success or any finding exists, so a clean structured run can be
reported as failure even when SARIF has zero results and the summary says
`Findings: 0`. `tests/consumer_integration.rs` also still encodes the stale
compile-error expectation (`exit 1`) on JSON/SARIF error paths, so spec, tests,
and runtime behavior drift together.

Aspen review churn showed a second gap: the runner emits findings, but it does
not emit one canonical run-status artifact bundle. Reviewers had to infer what
happened from summary text, ad-hoc logs, and shell-level capture instead of a
self-describing status file plus structured-output sidecars.

## Goals / Non-Goals

**Goals:**
- restore a deterministic `0` / `1` / `2` outcome contract in the runner
- make the human summary, machine-readable status, and SARIF invocation metadata agree on the same run classification
- give downstream automation an optional artifact bundle that captures command, summary, status, and structured output without shell scraping
- update consumer-integration tests so they enforce the same contract the runner ships

**Non-Goals:**
- adding new lints or changing existing lint severities
- replacing stdout/stderr as the default output surface when `--artifact-dir` is not used
- bundling downstream crate-test orchestration or OpenSpec evidence policy into the runner itself

## Decisions

### 1. Engine uses a typed run-outcome classifier

**Choice:** introduce a typed `RunOutcome` classifier in `cargo-tigerstyle/src/engine.rs` with explicit mappings for `clean`, `warning-only`, `lint-failure`, and `integration-failure`.

**Covers:** `Scenario: Clean structured run stays exit 0`; `Scenario: Warning-only structured run stays exit 0`; `Scenario: Error-level lint findings use exit 1`; `Scenario: Cargo compilation failure uses integration-failure exit code`; `Scenario: Terminated cargo child records terminated classification`

**Rationale:** the current boolean collapse (`!status.success() || !findings.is_empty()`) throws away the distinction between child-process failure and lint severity. A typed classifier makes the exit-code contract inspectable and reusable by the summary, status sidecar, and SARIF emission paths.

**Alternative considered:** keep the current boolean exit logic and special-case clean structured runs after SARIF emission. Rejected because it preserves the same hidden coupling and still leaves compile-failure vs lint-failure ambiguous.

**Implementation:** while parsing diagnostics, accumulate warning/error/autofixable counts separately from child cargo completion. The counters include only diagnostics already recognized as tigerstyle findings after lint-name metadata lookup, not arbitrary rustc/cargo compile errors. Severity comes from those matched tigerstyle diagnostics' `compiler-message.message.level`: `warning` stays in the warning bucket, and `error` enters the error bucket. After `child.wait()`, map those tigerstyle-only counts plus child status into `RunOutcome`, with `integration-failure` taking precedence over `lint-failure` whenever the cargo child is non-success, then derive the final process exit code (`0`, `1`, or `2`) from that enum instead of from a raw boolean.

**Boundary cases:** warning-only findings still exit `0`; signaled or no-exit-code child termination stays `integration-failure` even when no findings were emitted; cargo-child failure still wins over any earlier error-level lint findings; compile-failure paths still preserve valid structured stdout for the selected format.

### 2. `--artifact-dir` becomes the canonical sidecar surface

**Choice:** add `--artifact-dir <dir>` to `cargo-tigerstyle check` and persist deterministic sidecars: `status.json`, `command.txt`, `summary.txt`, and structured-output mirrors for `json` / `sarif` runs.

**Covers:** `Scenario: Artifact directory captures a structured run`; `Scenario: Artifact directory captures a JSON run`; `Scenario: Command line quoting is deterministic`; `Scenario: Artifact directory records integration failure cleanly`; `Scenario: Artifact directory captures a human run`; `Scenario: Artifact directory setup failure exits 2`; `Scenario: Sidecar write failure after emission exits 2`; `Scenario: Bundle shape mismatch is invalid`; `Scenario: Terminated cargo child records terminated classification`

**Rationale:** downstream reviewers and agents need a single run directory they can cite. A directory bundle is stronger than a lone status file because it keeps the command line, summary, and selected structured output together.

**Alternative considered:** add only `--status-json <path>`. Rejected because it still leaves the caller to invent filenames and capture rules for summary text and structured outputs.

**Implementation:** extend `cargo-tigerstyle/src/cli.rs` to parse `--artifact-dir` before `--`, then prepare the artifact directory before required sidecar writes begin. Buffer each machine-readable stdout document (`results.jsonl` or `results.sarif`) and the trailing summary in memory. When `--artifact-dir` is present, attempt the sidecar writes before emitting that buffered summary or buffered machine-readable stdout so any metadata that carries the final tigerstyle exit code is rendered only after the artifact-write attempt has settled. If artifact-directory setup fails before any required sidecar write starts, force the final outcome to `integration-failure` / exit `2`, emit the final summary and buffered stdout for that outcome, and leave no valid bundle at that path. Write the selected structured-output sidecar last for `json` / `sarif` runs. If a required sidecar write fails after some required sidecars already exist, force the final outcome to `integration-failure` / exit `2`, rerender the summary and machine-readable stdout from that final outcome, remove the selected structured-output sidecar best-effort so at least one required sidecar is absent, rewrite `command.txt`, `status.json`, and `summary.txt` best-effort, then emit stdout/stderr and leave the incomplete bundle invalid. `status.json` uses exact top-level keys `status`, `exit_code`, `output_format`, `total_findings`, `warning_findings`, `error_findings`, `autofixable_findings`, and `cargo_process_exit`. `status.json.exit_code` is an integer limited to `0`, `1`, or `2` and equals the final tigerstyle process exit code. `status.json.total_findings`, `status.json.warning_findings`, `status.json.error_findings`, and `status.json.autofixable_findings` are non-negative integers. `cargo_process_exit` is an object with exact keys `classification` and `code`: `classification` is `success` when the cargo child exits `0`, `exit-code` when the cargo child exits with a non-zero integer code, and `terminated` when the cargo child has no exit code; `code` is that integer exit code or `null` for `terminated`. `command.txt` is the exact top-level command line as one POSIX `sh`-compatible quoted line plus a trailing newline: arguments matching `[A-Za-z0-9._/:=@-]+` stay unquoted, and every other argument is single-quoted, with each embedded `'` escaped by closing the single-quoted string, inserting a double-quoted single quote, and reopening the single-quoted string. `summary.txt` is emitted from the same summary renderer used for stderr. `results.jsonl` is the exact newline-delimited JSON stdout stream for `--output-format json`, and `results.sarif` is the exact SARIF stdout document for `--output-format sarif`. A bundle is valid only when all required sidecars for its `output_format` are present and no extra sidecar file is present: `human` => only `command.txt`, `status.json`, `summary.txt`; `json` => only those files plus `results.jsonl`; `sarif` => only those files plus `results.sarif`.

**Boundary cases:** failed runs still write `status.json` and `summary.txt` once the engine reaches the final classification; human-mode rendered diagnostics may still stream during cargo parsing because they do not encode the final exit state; `human` mode may omit a structured-output mirror; an unusable `--artifact-dir` path before required sidecar writes begin returns `integration-failure` / exit `2` and callers treat that path as an invalid bundle target; if a later required sidecar write fails after some required sidecars were already written, the final process exit is `2`, the trailing summary and any emitted machine-readable stdout still reflect that final result, and the runner leaves at least one required sidecar absent so the bundle is invalid under the presence-based validity rule; existing stdout/stderr behavior stays unchanged when `--artifact-dir` is absent.

### 3. Summary and SARIF must carry the same status contract

**Choice:** extend the stderr summary with explicit status and per-severity counts, and extend SARIF with `runs[0].invocations[0]` plus run properties that mirror the same classification exposed in `status.json`.

**Covers:** `Scenario: SARIF carries invocation metadata`; `Scenario: Clean structured run stays exit 0`

**Rationale:** the same run should not look clean in one place and ambiguous in another. Self-describing SARIF matters for code-scanning and saved evidence; self-describing summaries matter for humans reading stderr directly.

**Alternative considered:** keep status metadata only in `status.json`. Rejected because standalone SARIF files and shell-only stderr captures would still require an external sidecar to explain the run outcome.

**Implementation:** update `cargo-tigerstyle/src/output.rs` so summaries render with this exact line order: `--- tigerstyle summary ---`, `Status: <status>`, `Findings: <count>`, `Warnings: <count>`, `Errors: <count>`, `Autofixable: <count>`, optional blank-line-separated `By crate:`, `By lint:`, and `Index:` sections for non-zero findings, then closing `---`. The `Status:` value is exactly `clean`, `warning-only`, `lint-failure`, or `integration-failure`. The SARIF builder emits `runs[0].invocations[0].commandLine`, `runs[0].invocations[0].exitCode`, and exact run-property keys `status`, `exitCode`, `totalFindings`, `warningFindings`, `errorFindings`, `autofixableFindings`, `cargoProcessExitClassification`, and `cargoProcessExitCode`. `runs[0].invocations[0].exitCode`, `runs[0].properties.exitCode`, and `status.json.exit_code` all mean the final tigerstyle process exit code; `runs[0].properties.totalFindings`, `runs[0].properties.warningFindings`, `runs[0].properties.errorFindings`, and `runs[0].properties.autofixableFindings` are non-negative integers; and `runs[0].properties.cargoProcessExitCode` mirrors `status.json.cargo_process_exit.code` from the underlying cargo child process.

**Boundary cases:** clean SARIF still emits a valid document with zero results and one invocation record; integration-failure SARIF stays valid JSON even when the underlying cargo check failed.

### 4. Consumer integration tests become the contract lock

**Choice:** update `tests/consumer_integration.rs` to assert the new exit-code contract, artifact-dir schema, and SARIF invocation metadata on both clean and compile-failure paths.

**Covers:** `Scenario: Clean structured run stays exit 0`; `Scenario: Warning-only structured run stays exit 0`; `Scenario: Error-level lint findings use exit 1`; `Scenario: Cargo compilation failure uses integration-failure exit code`; `Scenario: Temp-workspace launch keeps the consumer cwd`; `Scenario: Artifact directory captures a structured run`; `Scenario: Artifact directory captures a JSON run`; `Scenario: Command line quoting is deterministic`; `Scenario: Artifact directory records integration failure cleanly`; `Scenario: Artifact directory captures a human run`; `Scenario: SARIF carries invocation metadata`; `Scenario: Artifact directory setup failure exits 2`; `Scenario: Sidecar write failure after emission exits 2`; `Scenario: Bundle shape mismatch is invalid`; `Scenario: Terminated cargo child records terminated classification`

**Rationale:** this repo already has consumer-integration fixtures for warning-only and invalid-metadata paths. Extending those fixtures is the cheapest way to prevent another silent drift between spec and implementation.

**Alternative considered:** rely on unit tests around a new classifier helper. Rejected because CLI parsing, summary rendering, artifact writing, and SARIF output are all end-to-end behaviors.

**Implementation:** replace stale `exit 1` error-path assertions with `exit 2`, add artifact-dir fixture helpers that read saved sidecars, add dedicated warning-only fixtures that prove successful `json` and `sarif` artifact bundles exit `0`, add explicit assertions for summary status text plus SARIF invocation metadata, and make the artifact-bundle checks parse `command.txt` with `sh -n` while the CLI unit rail asserts the exact quoting literals. Refresh `README.md` and `docs/consumer-guide.md` so they both document the hardened exit codes, `--artifact-dir` bundle surface, and the supported temp-workspace launch path. The terminated-child rail stays deterministic by having the fake cargo helper honor `TS_KILL_SELF=1` and send `SIGKILL` to itself inside the repo's Unix/Linux nix devshell, which makes `ExitStatus::code()` report `None` without depending on external timing. Any consumer-integration helper that launches the CLI from a temp consumer workspace must keep the cwd at that temp workspace and invoke the binary via an explicit `--manifest-path /path/to/tigerstyle/cargo-tigerstyle/Cargo.toml` (or the already-built binary path) instead of `cargo run -p cargo-tigerstyle` from the temp workspace.

**Boundary cases:** warning-only fixtures must keep exiting `0` while the classifier is refactored.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Clean structured run stays exit 0`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/clean-sarif-status.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration clean_structured_run_stays_exit_zero -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test clean_structured_run_stays_exit_zero ... ok` and the saved assertions prove exit code `0`, the summary block uses the exact required line order with `Status: clean`, `Findings: 0`, `Warnings: 0`, `Errors: 0`, `Autofixable: 0`, and `results[].len() == 0`
- ordering / prerequisites:
  - land the `RunOutcome` classifier and clean-run fixture before saving the transcript
- cleanup / postcondition:
  - no temporary artifact dir remains outside the saved evidence path

- scenario or requirement:
  - `Scenario: Warning-only structured run stays exit 0`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/warning-only-status.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration warning_level_lint_findings_exit_with_code_zero -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test warning_level_lint_findings_exit_with_code_zero ... ok` and the saved assertions prove exit code `0`, the summary block uses the exact required line order with `Status: warning-only`, `Findings: 1`, `Warnings: 1`, `Errors: 0`, `Autofixable: 0`, and the non-zero `By crate:`, `By lint:`, and `Index:` sections in that order, and one valid JSON finding on stdout
- ordering / prerequisites:
  - land the warning-only fixture after the typed classifier is in place
- cleanup / postcondition:
  - no temporary artifact dir remains outside the saved evidence path

- scenario or requirement:
  - `Scenario: Error-level lint findings use exit 1`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/lint-failure-status.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration error_level_lint_findings_exit_with_code_one -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test error_level_lint_findings_exit_with_code_one ... ok` and the saved assertions prove exit code `1` and the summary block uses the exact required line order with `Status: lint-failure`, `Findings: 1`, `Warnings: 0`, `Errors: 1`, `Autofixable: 0`, and the non-zero `By crate:`, `By lint:`, and `Index:` sections in that order
- ordering / prerequisites:
  - land the error-level fixture after the typed classifier is in place
- cleanup / postcondition:
  - no temporary artifact dir remains outside the saved evidence path

- scenario or requirement:
  - `Scenario: Cargo compilation failure uses integration-failure exit code`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/error-path-exit-code.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration cargo_compilation_failure_uses_integration_failure_exit_code -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test cargo_compilation_failure_uses_integration_failure_exit_code ... ok` and the saved assertions prove exit code `2` for both JSON and SARIF compile-failure paths, JSON stdout stays valid JSON lines, SARIF stdout stays valid SARIF JSON with `integration-failure` metadata, and the summary block uses the exact required line order with `Status: integration-failure`
- ordering / prerequisites:
  - update the compile-failure fixture expectations after the classifier lands
- cleanup / postcondition:
  - no stale error-path fixture output remains in temporary dirs

- scenario or requirement:
  - `Scenario: Artifact directory captures a structured run`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-structured-run.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_captures_structured_run -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_captures_structured_run ... ok` and the saved assertions prove the artifact dir contains exactly `status.json`, `command.txt`, `summary.txt`, and `results.sarif`; `status.json` uses the exact top-level key set plus exact `cargo_process_exit.{classification,code}` keys and allowed values; `status.json.output_format == "sarif"`; `status.json.exit_code` equals the observed process exit code; parsed `status.json` classification/count fields agree with `summary.txt`; `command.txt` is the exact single shell-quoted command line plus trailing newline and parses cleanly under `sh -n`; `summary.txt` is a byte-for-byte copy of stderr's trailing summary block; `results.sarif` byte-matches stdout; and SARIF run properties expose the exact required key set with `exitCode` equal to the observed process exit code and `cargoProcessExitCode == status.json.cargo_process_exit.code`
- ordering / prerequisites:
  - implement `--artifact-dir` before saving the transcript
- cleanup / postcondition:
  - the test-owned temp directory is removed after the transcript is saved

- scenario or requirement:
  - `Scenario: Artifact directory captures a JSON run`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-json-run.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_captures_json_run -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_captures_json_run ... ok` and the saved assertions prove the artifact dir contains exactly `status.json`, `command.txt`, `summary.txt`, and `results.jsonl`; `status.json.output_format == "json"`; the count fields are non-negative integers matching the warning-only run; and `results.jsonl` byte-matches a valid newline-delimited JSON stdout stream
- ordering / prerequisites:
  - reuse the artifact-dir plumbing after the JSON sidecar filename and status schema are fixed
- cleanup / postcondition:
  - the test-owned temp directory is removed after the transcript is saved

- scenario or requirement:
  - `Scenario: Command line quoting is deterministic`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/command-line-quoting.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test -p cargo-tigerstyle cli::tests::command_line_quoting_matches_contract -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test cli::tests::command_line_quoting_matches_contract ... ok` and the saved assertions prove safe arguments stay unquoted, space-containing arguments are single-quoted, embedded single quotes use the specified close / double-quoted-single-quote / reopen form, and the empty argument renders as `''`
- ordering / prerequisites:
  - land the command-line renderer and its exact quoting test before saving the transcript
- cleanup / postcondition:
  - no temp artifact directory is needed for this renderer-only proof

- scenario or requirement:
  - `Scenario: Artifact directory records integration failure cleanly`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-error-path.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_records_integration_failure_cleanly -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_records_integration_failure_cleanly ... ok` and the saved assertions prove `status.json` reports `integration-failure` with the exact key set plus exact `cargo_process_exit.{classification,code}` keys and allowed values, `status.json.exit_code` equals the observed process exit code, `summary.txt` byte-matches stderr's trailing summary block, `results.jsonl` is the exact emitted JSON-lines artifact filename, `results.jsonl` byte-matches stdout, and the structured-output mirror remains valid JSON for the selected format
- ordering / prerequisites:
  - reuse the compile-failure fixture after `--artifact-dir` support lands
- cleanup / postcondition:
  - only the saved transcript remains under `evidence/`

- scenario or requirement:
  - `Scenario: SARIF carries invocation metadata`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/sarif-invocation.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration sarif_carries_invocation_metadata -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test sarif_carries_invocation_metadata ... ok` and the saved assertions prove `runs[0].invocations[0]` exists with the command line and exit code, `runs[0].invocations[0].exitCode` equals the observed process exit code, and run properties expose the exact required key set with the same status classification, tigerstyle exit code, and cargo-process classification values as `status.json`
- ordering / prerequisites:
  - land SARIF invocation metadata after the sidecar status schema is finalized so both use the same field names
- cleanup / postcondition:
  - no temporary SARIF fixture file remains outside the test tempdir

- scenario or requirement:
  - `Scenario: Artifact directory captures a human run`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-human-run.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_captures_human_run -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_captures_human_run ... ok` and the saved assertions prove the artifact dir contains exactly `status.json`, `command.txt`, and `summary.txt`; both structured-output sidecars are absent; `status.json.output_format == "human"`; `status.json.status == "warning-only"`; stdout stays empty; and `summary.txt` byte-matches the trailing stderr summary block with the expected warning-only summary/count lines after rendered diagnostics
- ordering / prerequisites:
  - reuse the artifact-dir plumbing after the human summary renderer is stable
- cleanup / postcondition:
  - no structured-output sidecar remains in the saved human-mode artifact dir

- scenario or requirement:
  - `Scenario: Artifact directory setup failure exits 2`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-setup-failure.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_setup_failure_exits_two -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_setup_failure_exits_two ... ok` and the saved assertions prove the process exits `2`, the trailing summary reports `integration-failure`, the emitted SARIF stdout reports `integration-failure` with `exitCode == 2`, and callers do not receive a valid artifact directory bundle from that unusable path
- ordering / prerequisites:
  - reuse the artifact-dir plumbing after the exact sidecar schema is in place
- cleanup / postcondition:
  - the failing artifact path remains an invalid bundle target and no reviewable sidecar directory is left behind

- scenario or requirement:
  - `Scenario: Bundle shape mismatch is invalid`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/partial-bundle-invalid.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration partial_artifact_bundle_is_invalid -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test partial_artifact_bundle_is_invalid ... ok` and the saved assertions prove a bundle with any missing required sidecar or any extra sidecar file is invalid for `human`, `json`, or `sarif`
- ordering / prerequisites:
  - land the required-file validity helper after the exact bundle file set is fixed
- cleanup / postcondition:
  - any temporary partial bundle directories are removed after the test

- scenario or requirement:
  - `Scenario: Sidecar write failure after emission exits 2`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/artifact-dir-mid-write-failure.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_mid_write_failure_exits_two_and_invalidates_bundle -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_mid_write_failure_exits_two_and_invalidates_bundle ... ok` and the saved assertions prove a required sidecar write fails after some required sidecars already exist, the final process exit is `2`, the emitted SARIF stdout still reports `integration-failure` with `exitCode == 2`, `status.json` is rewritten to the same final status, the selected structured-output sidecar is absent, and the resulting bundle is invalid
- ordering / prerequisites:
  - reuse the artifact-dir plumbing after the required-file validity rule is in place
- cleanup / postcondition:
  - the saved artifact dir remains present only as an invalid incomplete bundle

- scenario or requirement:
  - `Scenario: Terminated cargo child records terminated classification`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/terminated-status.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration terminated_child_artifact_records_terminated_classification -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test terminated_child_artifact_records_terminated_classification ... ok` and the saved assertions prove the fake cargo helper self-terminates under `TS_KILL_SELF=1`, the final process exit is `2`, `status.json.cargo_process_exit.classification == "terminated"`, and `status.json.cargo_process_exit.code == null`
- ordering / prerequisites:
  - reuse the artifact-dir status sidecar path after the terminated-child classifier is wired
- cleanup / postcondition:
  - no extra artifact bundle is left outside the saved evidence path

- scenario or requirement:
  - `Scenario: Temp-workspace launch keeps the consumer cwd`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/temp-workspace-launch.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration temp_workspace_launch_uses_built_binary_and_preserves_consumer_cwd -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test temp_workspace_launch_uses_built_binary_and_preserves_consumer_cwd ... ok` and the saved assertions prove the launcher path is an explicit built binary, the run starts from a nested consumer-workspace cwd, and `[workspace.metadata.tigerstyle]` defaults still resolve from that consumer workspace
- ordering / prerequisites:
  - run after the temp-workspace launch helper path is pinned to the built binary or explicit `--manifest-path` contract
- cleanup / postcondition:
  - no temp launcher state remains outside the test fixture root

- scenario or requirement:
  - documentation contract for `README.md` and `docs/consumer-guide.md`
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/docs-contract.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-04-21-harden-check-status-artifacts/check-docs-contract.py`
- exact expected output / acceptance signal:
  - the transcript prints `README.md: ok`, `docs/consumer-guide.md: ok`, and `docs contract: passed`, proving the docs mention the hardened exit codes, `--artifact-dir` bundle surface, and the supported temp-workspace `--manifest-path` launch path
- ordering / prerequisites:
  - run after the doc edits land in `README.md` and `docs/consumer-guide.md`
- cleanup / postcondition:
  - the helper script remains under the active change path as the exact replay path for the saved transcript

- scenario or requirement:
  - human/oracle checkpoint for the full change
- artifact path:
  - `openspec/changes/archive/2026-04-21-harden-check-status-artifacts/evidence/design-gate.txt`
- command / helper:
  - `openspec_gate stage=design change=harden-check-status-artifacts`
- exact expected output / acceptance signal:
  - the transcript reports a passing design-stage gate result with no blocker findings about exit-code ambiguity, artifact schema ambiguity, or missing scenario coverage
- ordering / prerequisites:
  - run after proposal, consumer-integration delta spec, and design are all finalized
- cleanup / postcondition:
  - the saved transcript references the active change path only

## Risks / Trade-offs

- more CLI surface area -> keep `--artifact-dir` optional and preserve existing stdout/stderr defaults when it is absent
- status metadata drift across summary, sidecar, and SARIF -> drive all three from one `RunOutcome` structure and one count aggregation path
