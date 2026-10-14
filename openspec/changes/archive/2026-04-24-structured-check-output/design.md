## Context

`cargo tigerstyle check` currently runs `cargo check` as a child process with Dylint environment variables. Diagnostics flow through rustc's standard rendering pipeline to stderr. The engine has no visibility into individual diagnostics — it only sees the child process exit code.

To produce structured output, we need to intercept rustc's diagnostic stream. Rustc already supports `--error-format=json` which emits one JSON object per diagnostic on stdout (when used via `cargo check --message-format=json`). This is the same mechanism `cargo clippy --message-format=json` uses.

## Goals / Non-Goals

**Goals:**
- Machine-readable output for LLM-driven triage and fix workflows
- Per-run summary for human progress tracking
- Stable finding identifiers for granular task assignment
- Zero regression in default human-readable mode

**Non-Goals:**
- Persistent finding databases or cross-run deduplication
- IDE/LSP integration
- Custom diagnostic rendering (we reuse rustc's JSON format as the ingestion source)

## Decisions

### 1. Capture diagnostics via `cargo check --message-format=json`

**Choice:** When `--output-format` is `json` or `sarif`, pass `--message-format=json` to the inner `cargo check` invocation and capture stdout from the child process. Parse each JSON line, filter to tigerstyle lint diagnostics (by lint name prefix), enrich with index/group/principle metadata from `DIAGNOSTIC_METADATA`, and re-emit in the requested format.

**Rationale:** This reuses rustc's existing structured output rather than building a custom diagnostic sink. The `--message-format=json` format is stable and well-documented. Cargo emits `compiler-message` objects that contain the full `rustc` diagnostic JSON, including spans, suggestions, and lint names.

**Alternative:** Build a custom `rustc_driver` diagnostic emitter that writes structured data directly. Rejected because it couples tightly to rustc internals and would require changes to the lint library itself, not just the runner.

**Implementation:**
- In `engine.rs`, **all modes** (including `human`) pass `--message-format=json` to the `cargo check` args and capture child stdout via `Stdio::piped()`.
- Parse each line as a `cargo` message; filter for `compiler-message` kind where the lint code starts with `tigerstyle::` or matches a bare lint name from `DIAGNOSTIC_METADATA`.
- Assign finding indices (`F1`, `F2`, ...) in order of appearance.
- In `human` mode, for each `compiler-message`, print the `message.rendered` field to stderr. This field contains the exact human-readable diagnostic output that rustc would have printed, including colors when stderr is a TTY. This preserves the current visual experience while giving the engine full access to structured diagnostic data for summary computation.
- For non-diagnostic cargo messages (`compiler-artifact`, `build-script-executed`, `build-finished`): in `human` mode, the engine reconstructs the familiar `Compiling ...` and `Finished ...` banners from the JSON `compiler-artifact` and `build-finished` messages. Progress bar rendering is lost (cargo suppresses it under `--message-format=json`); this is an accepted behavioral difference documented in the spec.
- In `json` and `sarif` modes, non-diagnostic messages are silently consumed (not emitted to stdout or stderr).

**Lint name filtering:** Dylint-based lints currently emit bare lint names in the `message.code.code` field (e.g., `no_unwrap`, not `tigerstyle::no_unwrap`). The filter accepts any `code.code` value that matches a `lint_name` entry in `DIAGNOSTIC_METADATA`. If tool-scoped names are added in the future, the filter also accepts `code.code` values starting with `tigerstyle::` and strips the prefix for lookup. This handles the mixed bare/tool-scoped state documented in the napkin without requiring changes to the lint library.

### 2. New `output.rs` module for format rendering

**Choice:** Add `cargo-tigerstyle/src/output.rs` containing the structured output types, JSON serialization, SARIF document builder, and summary renderer.

**Rationale:** Keeps `engine.rs` focused on process orchestration. The output module owns the `Finding` struct, format-specific serialization, and summary computation.

**Implementation:**

```rust
// cargo-tigerstyle/src/output.rs

#[derive(Serialize)]
pub struct Finding {
    pub version: u32,                  // schema version, initially 1
    pub index: String,                 // "F1", "F2", ...
    pub lint: String,                  // "no_unwrap"
    pub group: String,                 // "tigerstyle::safety"
    pub severity: String,              // "warning" or "error"
    pub file: String,                  // relative path
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
    pub applicability: Option<String>,
    pub principle: String,             // from DIAGNOSTIC_METADATA, or "unknown"
    pub crate_name: String,            // from cargo compiler-message target.name
}
```

**`DIAGNOSTIC_METADATA` lookup:** The existing `DIAGNOSTIC_METADATA` constant in `src/diagnostics.rs` already maps every registered lint to its `domain_group` and `principle`. The `output.rs` module builds a `HashMap<&str, &LintDiagnosticMeta>` keyed by `lint_name` at engine startup. For any lint not found in the map, `group` defaults to `"tigerstyle::unknown"` and `principle` defaults to `"unknown"`. All currently shipped lints have entries, so this fallback is defensive only.

**SARIF version source:** `tool.driver.version` is set to `env!("CARGO_PKG_VERSION")` from cargo-tigerstyle's `Cargo.toml`.

**SARIF `shortDescription` source:** Each rule's `shortDescription.text` is set to the `guidance` field from `DIAGNOSTIC_METADATA` (e.g., `"split compound assertions so the failing clause is obvious"` for `compound_assertion`).
```

### 3. Summary always on stderr

**Choice:** Print the summary to stderr in all modes, including `json` and `sarif` where stdout carries structured data.

**Rationale:** Keeps stdout clean for piping (`cargo tigerstyle check --output-format json | jq ...`) while still giving humans a quick glance. An LLM consuming stdout gets pure JSON; a human watching the terminal sees the summary.

**Summary format:**

The summary block is delimited by `--- tigerstyle summary ---` and `---` lines for machine extraction. Finding indices are listed per-finding in the summary so a human or LLM can reference specific items:

```
--- tigerstyle summary ---
Findings: 15 (4 autofixable)

By crate:
  aspen-core        8
  aspen-dns         4
  aspen-hlc         3

By lint:
  no_unwrap         6
  no_panic          4
  unbounded_channel 3
  function_length   2

Index:
  F1  no_unwrap         aspen-core  src/connection.rs:42
  F2  function_length   aspen-core  src/router.rs:15
  F3  unbounded_channel aspen-dns   src/resolver.rs:28
  ...
---
```

In `human` mode, finding indices do NOT prefix the inline rustc diagnostic output (preserving the "identical diagnostic stream" guarantee). They appear only in the trailing summary index table.

For zero findings:

```
--- tigerstyle summary ---
Findings: 0
---
```

### 4. Finding index assignment is order-of-appearance

**Choice:** Indices are assigned sequentially as diagnostics arrive from the cargo subprocess. They are not stable across runs if the diagnostic order changes (e.g., due to parallel compilation).

**Rationale:** True content-addressable IDs (hashing file+line+lint) are more complex and still break on code movement. Order-based indices are simple, sufficient for single-run LLM workflows, and match how humans read output top-to-bottom.

### 5. SARIF built by hand, no external crate

**Choice:** Emit SARIF v2.1.0 JSON using `serde_json::Value` construction rather than depending on a SARIF crate.

**Rationale:** The SARIF schema is straightforward for our use case (tool + rules + results with locations). Adding a crate dependency for this gains little and complicates the Nix build. The output is a single JSON document written after all findings are collected.

## Golden Output Artifacts

The following files define the exact user-visible output contract:

- `golden/json-example.jsonl` — three-finding JSON-lines example with all required fields
- `golden/sarif-example.json` — complete SARIF v2.1.0 document for the same three findings
- `golden/summary-example.txt` — summary block format for a multi-crate run

Integration tests MUST validate that real output structurally matches these golden artifacts (field presence, types, and delimiters).

## Risks / Trade-offs

**[Diagnostic ordering non-determinism]** Cargo may emit diagnostics in different orders across runs due to parallel compilation. Finding indices like `F3` may refer to different findings between runs. → Acceptable for single-run workflows. Document that indices are per-run, not persistent.

**[Cargo message format stability]** We depend on `cargo`'s `--message-format=json` output shape. → This format has been stable since Cargo 1.30+ and is used by `rust-analyzer`, `cargo-clippy`, and many other tools. Low risk.

**[Performance of stdout capture]** Piping and parsing child stdout adds overhead vs. pass-through. → Negligible for typical check runs. Active in all modes (including `human`) since the engine needs structured data for summary computation.

**[Missing lint name in cargo JSON]** Cargo's JSON `compiler-message` includes the lint name under `message.code.code`. For Dylint-based lints, this field contains the bare lint name. If rustc changes this field's format, our filtering breaks. → Pin on the known format and add a test.

**[Error path handling]** When `cargo check` emits a compilation error (not a lint diagnostic), the engine must still finalize output cleanly. For JSON mode this means no partial line on stdout. For SARIF mode this means the document is closed with whatever results were collected. The summary always prints regardless. Exit code follows the underlying cargo exit.

**[Color / TTY detection]** When stderr is a TTY and the user has not explicitly set `--color`, the engine passes `--color=always` to the inner `cargo check` so that the `message.rendered` field includes ANSI color codes. When stderr is not a TTY (piped), `--color` defaults to cargo's auto behavior (no color). Users can override with `-- --color=never` or `-- --color=always` via the passthrough args.

**[Passthrough `--message-format` conflict]** If the user passes `--message-format` via `-- --message-format=...`, it conflicts with the engine's injected `--message-format=json`. The engine MUST detect this and exit with code 2 and an error message explaining that `--message-format` is managed by `--output-format`.

**[`--output-format` flag position]** `--output-format` is parsed before the `--` separator as a cargo-tigerstyle flag. Anything after `--` is passthrough to `cargo check`. If `--output-format` appears after `--`, it is treated as a cargo arg (not recognized by cargo-tigerstyle).
