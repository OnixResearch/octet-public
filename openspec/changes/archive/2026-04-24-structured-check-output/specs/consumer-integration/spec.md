## ADDED Requirements

### Requirement: JSON Diagnostic Output
The system MUST support a `--output-format json` flag that emits one JSON object per diagnostic line to stdout, while rustc's human-readable output continues on stderr.

Each JSON object MUST contain the following fields with the specified types:
- `index`: string — finding identifier (`F1`, `F2`, ...) unique within the run
- `lint`: string — bare lint name (e.g., `no_unwrap`)
- `group`: string — domain group, one of `tigerstyle::safety`, `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`, `tigerstyle::concurrency`, or `tigerstyle::unknown` for unrecognized lints. This is an output-side label resolved from a static lookup table (`DIAGNOSTIC_METADATA` in `src/diagnostics.rs`), not dependent on Dylint/rustc group registration
- `severity`: string — effective severity, one of `warning`, `error`
- `file`: string — relative file path from workspace root
- `line`: integer — 1-based line number
- `column`: integer — 1-based column number
- `message`: string — primary diagnostic message text
- `suggestion`: string or null — suggested replacement text if the lint emits one
- `applicability`: string or null — one of `MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, or null
- `principle`: string — Tiger Style principle name sourced from `DIAGNOSTIC_METADATA` in `src/diagnostics.rs` (e.g., `Assertion Quality`, `Function Length`). If a lint fires that is not registered in `DIAGNOSTIC_METADATA`, the value MUST be `"unknown"`
- `crate_name`: string — name of the crate being checked, extracted from the `target.name` field in cargo's `compiler-message` JSON wrapper
- `version`: integer — schema version, initially `1`; incremented on breaking field changes

#### Scenario: LLM parses JSON findings into granular tasks
- GIVEN a workspace with findings from multiple lints across multiple crates
- WHEN `cargo tigerstyle check --output-format json` completes
- THEN stdout contains one valid JSON object per line, each parseable with `serde_json::from_str`, and every object has all required fields

#### Scenario: JSON output coexists with human stderr
- GIVEN `--output-format json` is specified
- WHEN the check runs
- THEN JSON findings appear on stdout (one per line), and stderr contains only the trailing summary block (no human-readable diagnostics in `json` mode, since stdout carries the structured data)

### Requirement: SARIF Output
The system MUST support a `--output-format sarif` flag that emits a SARIF v2.1.0 document to stdout.

The SARIF document MUST include:
- `$schema` set to the SARIF v2.1.0 JSON schema URI
- `version` set to `"2.1.0"`
- `runs` array with exactly one run object containing:
  - `tool.driver.name` set to `"cargo-tigerstyle"`
  - `tool.driver.version` set to the cargo-tigerstyle version
  - `tool.driver.rules[]` with one entry per distinct lint that fired, each with `id` (bare lint name), `shortDescription`, and a `properties.group` field
  - `results[]` with one entry per finding, each containing:
    - `ruleId` set to the bare lint name
    - `level` set to `"warning"` or `"error"`
    - `message.text` matching the primary diagnostic message
    - `locations[0].physicalLocation` with `artifactLocation.uri` (relative file path), `region.startLine`, and `region.startColumn`
    - `properties.findingIndex` set to the finding index string (e.g., `"F3"`)

#### Scenario: GitHub Code Scanning ingestion
- GIVEN a workspace with findings
- WHEN `cargo tigerstyle check --output-format sarif` completes
- THEN the stdout output is valid JSON conforming to SARIF v2.1.0 structure with `$schema`, `version`, `runs[0].tool.driver.name`, `runs[0].tool.driver.rules[]`, and `runs[0].results[]` fields present. Validation is structural spot-checks in Rust integration tests, not full JSON Schema validation

### Requirement: Run Summary
The system MUST print a closing summary to stderr at the end of every check run, regardless of output format.

The summary MUST include:
- `--- tigerstyle summary ---` opening delimiter and `---` closing delimiter for machine extraction
- total finding count
- per-crate finding count (sorted descending by count)
- per-lint finding count (sorted descending by count)
- count of autofixable findings (those with `MachineApplicable` suggestions)
- per-finding index table listing each finding's index, lint, crate, and file:line

#### Scenario: Summary after noisy run
- GIVEN a workspace with 15 findings across 3 crates from 4 different lints
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary block showing crate counts, lint counts, total, and autofixable count

#### Scenario: Summary for clean run
- GIVEN a workspace with zero findings
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary showing `Findings: 0` (no autofixable parenthetical for zero findings)

### Requirement: Error Path Behavior
The system MUST produce valid output on stdout even when the underlying `cargo check` encounters compilation errors (exit code 2 path).

- In `json` mode: any findings emitted before the error MUST appear as valid JSON lines; no partial/malformed JSON on stdout
- In `sarif` mode: the SARIF document MUST still be a valid JSON document with whatever findings were emitted before the error
- In all modes: the summary MUST still print on stderr, reflecting only the findings that were successfully emitted
- The exit code MUST be `2` when the underlying cargo check fails, regardless of output format

#### Scenario: Compilation error during JSON output
- GIVEN a workspace with a syntax error in one crate and lint findings in another
- WHEN `cargo tigerstyle check --output-format json` runs
- THEN stdout contains valid JSON lines for findings emitted before the error, stderr contains the compilation error, and exit code is 2

#### Scenario: Compilation error during SARIF output
- GIVEN a workspace with a compilation error
- WHEN `cargo tigerstyle check --output-format sarif` runs
- THEN stdout is a valid SARIF JSON document (possibly with zero results), and exit code is 2

### Requirement: Finding Index
The system MUST assign each finding a deterministic index (`F1`, `F2`, ...) based on order of appearance in the diagnostic stream.

The index MUST be:
- unique within a single run
- monotonically increasing
- assigned by order of arrival from the cargo subprocess diagnostic stream (not sorted)
- present in all output formats (JSON field, SARIF property, and prefixed in human summary)

Indices are run-scoped only and NOT stable across invocations. In multi-crate workspaces, cargo may emit diagnostics in different orders due to parallel compilation, so the same finding may receive different indices between runs.

#### Scenario: LLM references specific finding by index
- GIVEN a run that emits 10 findings
- WHEN an LLM reads the JSON output
- THEN it can reference `F3` and the index maps to exactly one finding with a specific file, line, and lint

### Requirement: Output Format Flag
The system MUST accept `--output-format {human,json,sarif}` on the `check` subcommand.

The default MUST be `human`, which preserves the current pass-through behavior identically.

#### Scenario: Default behavior unchanged
- GIVEN no `--output-format` flag is specified
- WHEN `cargo tigerstyle check` runs
- THEN the lint diagnostic messages on stderr are identical to the current behavior (rustc human-readable diagnostics with colors preserved on TTY), exit code is unchanged, and a new trailing summary block is appended to stderr after the diagnostic stream. Note: cargo progress/build-status output may differ slightly since the engine uses `--message-format=json` internally

#### Scenario: Invalid format rejected
- GIVEN `--output-format xml` is specified
- WHEN `cargo tigerstyle check` runs
- THEN the tool exits with code 2 and an error message listing valid formats

#### Scenario: Conflicting --message-format rejected
- GIVEN `cargo tigerstyle check -- --message-format=short` is specified
- WHEN the tool runs
- THEN the tool exits with code 2 and an error message explaining that `--message-format` is managed by `--output-format`
