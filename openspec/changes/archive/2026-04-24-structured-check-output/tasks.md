## Phase 1: Implementation

- [x] I1 Add `--output-format {human,json,sarif}` flag to `cargo-tigerstyle/src/cli.rs` with `human` as the default; `--output-format` is parsed before the `--` separator
- [x] I5 Update `cargo-tigerstyle/src/main.rs` to thread the output format through to the engine
- [x] I2 Add `cargo-tigerstyle/src/output.rs` with `Finding` struct (including `version: 1`), JSON-lines emitter, SARIF document builder, and summary renderer
- [x] I3 Update `cargo-tigerstyle/src/engine.rs` to capture child stdout via `--message-format=json` in **all modes** (including `human`); parse `compiler-message` objects; filter to tigerstyle lints; assign finding indices; delegate to `output.rs`
- [x] I9 In all modes, inject `--color=always` when stderr is a TTY and the user has not overridden color via passthrough args
- [x] I8 In `human` mode, print each diagnostic's `message.rendered` to stderr and reconstruct `Compiling`/`Finished` banners from cargo JSON `compiler-artifact` and `build-finished` messages; in `json`/`sarif` modes, suppress human diagnostic output on stderr
- [x] I4 Wire summary printing to stderr at end of every run in all modes (including `human`), using the delimited `--- tigerstyle summary ---` format from design, including the per-finding index table
- [x] I6 Handle error paths: when `cargo check` exits with code 2, finalize JSON/SARIF output cleanly (no partial lines, valid SARIF document), still print summary, propagate exit code 2
- [x] I7 Detect `--message-format` in passthrough args and exit with code 2 and error message
- [x] I10 Commit the three golden output artifacts (`golden/json-example.jsonl`, `golden/sarif-example.json`, `golden/summary-example.txt`) already produced at design stage

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "LLM parses JSON findings into granular tasks" with a workspace fixture that produces multiple lint findings; parse JSON stdout and assert all required fields present (including `version: 1`); compare structural shape against `golden/json-example.jsonl`; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/json-output.txt`
- [x] V2 Verify scenario "JSON output coexists with human stderr" by running with `--output-format json` and asserting stdout is JSON-lines while stderr contains only the trailing summary block (no human diagnostics); save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/json-stderr-coexist.txt`
- [x] V3 Verify scenario "GitHub Code Scanning ingestion" by validating SARIF stdout against required structural fields (`$schema`, `version`, `runs[0].tool.driver.name`, `runs[0].tool.driver.rules[]`, `runs[0].results[].properties.findingIndex`); compare shape against `golden/sarif-example.json`; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/sarif-output.txt`
- [x] V4 Verify scenario "Summary after noisy run" by running against a workspace with multiple findings and asserting the summary block format on stderr matches the structure in `golden/summary-example.txt` (delimiters, per-crate counts, per-lint counts, index table); save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/summary-noisy.txt`
- [x] V5 Verify scenario "Summary for clean run" by running against a clean workspace and asserting `Findings: 0` in summary (no autofixable parenthetical); save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/summary-clean.txt`
- [x] V6 Verify scenario "LLM references specific finding by index" by parsing JSON output and asserting each finding has a unique `F<n>` index and the sequence is monotonically increasing; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/finding-index.txt`
- [x] V7 Verify scenario "Default behavior unchanged" by running with no `--output-format` flag and asserting: (a) lint diagnostics appear on stderr in human-readable format, (b) a trailing summary block is appended to stderr, (c) no output on stdout, (d) exit code matches expected; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/default-unchanged.txt`
- [x] V8 Verify scenario "Invalid format rejected" by running with `--output-format xml` and asserting exit code 2 with error message; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/invalid-format.txt`
- [x] V9 Verify scenario "Compilation error during JSON output" with a workspace containing a syntax error; assert stdout has valid JSON lines and exit code is 1; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/error-path-json.txt`
- [x] V10 Verify scenario "Compilation error during SARIF output" with a workspace containing a compilation error; assert stdout is valid SARIF JSON and exit code is 1; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/error-path-sarif.txt`
- [x] V11 Verify scenario "Conflicting --message-format rejected" by running with `-- --message-format=short` and asserting exit code 2; save output at `openspec/changes/archive/2026-04-24-structured-check-output/evidence/message-format-conflict.txt`

## Change Status
- Status: complete
- Completed: I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11

## Verification Coverage

- `Scenario: LLM parses JSON findings into granular tasks` -> `V1` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/json-output.txt`
- `Scenario: JSON output coexists with human stderr` -> `V2` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/json-stderr-coexist.txt`
- `Scenario: GitHub Code Scanning ingestion` -> `V3` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/sarif-output.txt`
- `Scenario: Summary after noisy run` -> `V4` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/summary-noisy.txt`
- `Scenario: Summary for clean run` -> `V5` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/summary-clean.txt`
- `Scenario: LLM references specific finding by index` -> `V6` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/finding-index.txt`
- `Scenario: Default behavior unchanged` -> `V7` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/default-unchanged.txt`
- `Scenario: Invalid format rejected` -> `V8` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/invalid-format.txt`
- `Scenario: Compilation error during JSON output` -> `V9` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/error-path-json.txt`
- `Scenario: Compilation error during SARIF output` -> `V10` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/error-path-sarif.txt`
- `Scenario: Conflicting --message-format rejected` -> `V11` -> `openspec/changes/archive/2026-04-24-structured-check-output/evidence/message-format-conflict.txt`
