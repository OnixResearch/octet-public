## Why

`cargo tigerstyle check` currently delegates to `cargo check` and passes through raw rustc diagnostic output on stderr. This output is:

1. **Unparseable without heuristics.** Warnings are interleaved free-form text. An LLM or script must regex-parse rustc's human-readable format to extract individual findings.
2. **No summary.** After a run with 40+ warnings across 8 crates, there is no roll-up showing which lints fired, how many times, or which crates are clean vs. noisy.
3. **No stable finding identifiers.** An LLM working through findings one-by-one cannot reference "finding #7" — it has to re-describe the file, line, and lint each time.
4. **No granular task decomposition.** Without structured data, an LLM cannot generate a prioritized work plan (e.g., "fix 3 `no_unwrap` in `aspen-core`, then 1 `unbounded_channel` in `aspen-dns`").

This change adds structured output modes to `cargo tigerstyle check` so both humans and LLMs can track, triage, and address findings granularly.

**Parsing strategy:** `cargo tigerstyle check` will pass `--message-format=json` to the underlying `cargo check` invocation and parse rustc's JSON diagnostic objects from the child process stdout. This is the same mechanism used by `cargo clippy --message-format=json` and `rust-analyzer`. Dylint-based lints flow through rustc's standard diagnostic pipeline, so their JSON output includes lint name, spans, suggestions, and applicability — all fields needed for structured output.

## What Changes

- **JSON output mode**: `cargo tigerstyle check --output-format json` emits one JSON object per diagnostic line on stdout, with lint name, group, severity, file, line, column, message, suggestion text, and applicability tier.
- **Summary report**: Every run prints a closing summary to stderr with per-crate and per-lint counts, autofixable count, and a suggested next-action ordering.
- **SARIF output mode**: `cargo tigerstyle check --output-format sarif` emits a SARIF v2.1.0 document for integration with GitHub Code Scanning, VS Code SARIF Viewer, and other tooling.
- **Finding index**: In all output modes, each finding gets a deterministic index (`F1`, `F2`, ...) scoped to the run, so an LLM or human can say "address F3" unambiguously.
- **Summary in all modes**: Every run (including `human` mode) prints a closing summary to stderr. This is new behavior added to all modes — `human` mode preserves the current diagnostic stream but appends the summary.

## Capabilities

### New Capabilities
- `structured-json-output`: Machine-readable JSON-lines diagnostic stream on stdout
- `structured-sarif-output`: SARIF v2.1.0 static analysis report on stdout
- `run-summary`: Per-crate and per-lint finding counts printed at end of every run
- `finding-index`: Stable per-run finding identifiers for granular task reference

### Modified Capabilities
- `check-exit-code`: Exit code semantics unchanged (0 = clean, 1 = findings, 2 = error)

## Impact

- **Files**: `cargo-tigerstyle/src/engine.rs`, `cargo-tigerstyle/src/cli.rs`, `cargo-tigerstyle/src/main.rs`, new `cargo-tigerstyle/src/output.rs`
- **APIs**: CLI gains `--output-format {human,json,sarif}` flag; default is `human` (current behavior)
- **Dependencies**: `serde_json` (already in tree), `serde` (already in tree); no new crates required for SARIF (hand-built to spec)
- **Testing**: New integration tests for JSON and SARIF output parsing; UI snapshot tests for summary format
- **Backwards compatibility**: Default behavior (`--output-format human`) is identical to current output. No breaking changes.

## Verification Expectations

- Integration tests parse JSON stdout and assert all required fields and types
- SARIF output validated against required structural fields (`$schema`, `version`, `runs[].tool.driver`, `runs[].results[]`)
- Summary format asserted by snapshot comparison
- Default `human` mode diffed against current behavior to prove no regression
- Golden output artifacts (JSON example, SARIF document, summary snapshot) produced at design stage

## Scope

### In scope
- JSON-lines output with one object per diagnostic
- SARIF v2.1.0 output document
- Closing summary table (always printed to stderr in all modes)
- Finding index assignment
- `--output-format` CLI flag

### Out of scope
- LSP / language server integration
- Interactive TUI dashboard
- Historical trend tracking across runs
- IDE plugin development
- HTML report generation
- `--fix` interaction with `--output-format` (no `--fix` subcommand exists today)
