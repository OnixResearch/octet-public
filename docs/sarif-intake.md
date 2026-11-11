# Local SARIF intake

Octet imports a bounded SARIF 2.1.0 subset from an explicit local file. Import mode never runs CodeQL, Semgrep, or a hosted scanner. A separate, opt-in execution mode runs a checked producer profile locally through Bounded Exec and then feeds its report through the same importer.

## Import

Select one checked producer profile:

```console
cargo octet evidence collect \
  --rail sarif-import \
  --profile codeql-local \
  --config standards/sarif/profiles.json \
  --input target/codeql.sarif \
  --artifact-dir artifacts/sarif-codeql
```

Use `semgrep-local` for a Semgrep report. Octet rejects `--run` and `--command` for the `-local` intake profiles, and it rejects a `-exec-local` producer profile on the import path.

The profile binds the driver name, supported version, required rule identity, source-level mapping, local path policy, evidence tier, and resource limits. The checked JSON profile has BLAKE3 identity `b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7`.

## Bounded local execution

Execution is opt-in and uses separate checked producer profiles. The shipped profiles pin Semgrep `1.180.0` and CodeQL `2.27.2`; the CodeQL query pack is `codeql/rust-queries@0.1.46`. Real Semgrep `1.180.0` and CodeQL `2.27.2` runs against the fixture crates under `standards/sarif/fixtures/` produced the retained clean and violating reports. The unit tests otherwise use an explicitly labeled test-only executable for failure-path coverage. Semgrep 1.180.0 emits the driver name `Semgrep OSS`; the importer accepts that spelling for the checked `Semgrep` profile.

```console
cargo octet evidence collect \
  --rail sarif-import \
  --profile semgrep-exec-local \
  --config standards/sarif/producers.json \
  --workspace-root . \
  --run \
  --artifact-dir artifacts/semgrep-exec
```

Use `codeql-exec-local` for CodeQL. The rail rejects `--input`, `--command`, `--baseline`, `--external-report`, `--parent-artifact`, `--debug-companion`, `--function-address-artifact`, and `--requirement-refs` in this mode: the checked profile, not caller text, is the only execution authority.

The checked producer profile binds the declared and observed executable version, the executable file's BLAKE3 identity, the checked rule-bundle bytes or the pinned query-pack set, the selected workspace-relative paths, extensions and exclusion globs, the explicitly unsupported Rust features, the fixed argv each step may use, resource limits, and non-claims. `producers.ncl` is the human-authored source; `producers.json` is its deterministic Nickel export with BLAKE3 identity `b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd`.

Octet runs every step through Bounded Exec in its own process group with the profile's wall-clock deadline and per-stream caps. Each execution writes `sarif-producer-execution.json` next to the receipt. That versioned receipt binds:

- the producer profile and intake profile identities;
- the resolved executable path, executable BLAKE3 identity, declared version, and observed `semgrep --version` or `codeql version --format=json` version;
- the rule-bundle identity or the CodeQL query-pack pins;
- the exact per-step argv, exit codes, and step outcomes;
- the source snapshot BLAKE3 identity and its sorted `path`/`content` identity rows over the declared files and exclusions;
- the declared Rust scope, including explicit unsupported features;
- the resource limits, process outcome, and raw report BLAKE3 identity;
- for CodeQL, the database creation and analysis steps separately, plus the database binding.

The receipt's `bounded_log_identity` is the execution receipt artifact's BLAKE3 identity, and provenance records that artifact so artifact verification rechecks the exact retained bytes. The raw report and its normalized projection keep their existing identities and provenance linkage.

### CodeQL database binding

CodeQL staging lives under `<workspace>/.octet/sarif-producer/codeql-exec-local/`. The first successful `codeql database create` records an Octet-owned `octet-sarif-database-binding.json` inside the database directory binding the source snapshot, language, build mode, query packs, and executable identity. Analysis rereads that binding and fails when the database is stale, was created for a different source snapshot, build configuration, query pack set, or executable, or has no Octet binding at all.

### Outcomes

An executed receipt reports one of these outcomes:

- `completed`: the process exited successfully, the report is valid, every finding path is inside the bound snapshot, and the source snapshot did not change during the scan.
- `failed`: the scan completed but its report is malformed, references files outside the declared scope, or the source changed during the scan.
- `unsupported`: the executable is missing, or the declared scope selects no supported source file.
- `tool_error`: the executable resolved but reported another version, a fixed step exited nonzero, or the report disagrees with the executed version or query packs.
- `timeout`: Bounded Exec reached the profile deadline.
- `incomplete_analysis`: Bounded Exec hit a stream cap, or the scanner exited successfully without a report.

An empty SARIF result set is clean only when the outcome is `completed` and the bound snapshot and scope are established. Missing tools, failed steps, timeouts, truncated output, and scope mismatches never produce a passing clean receipt.

## Supported subset

A report contains one local run. The run contains a declared driver, a `tool.driver.rules` array, bounded results, messages, and one physical location per result. The run's `results` must be an array: SARIF treats absent or `null` results as a tool that failed to start, so Octet rejects that run, and it rejects a run whose `invocations` report `executionSuccessful: false`. `tool.extensions`, `artifacts`, `invocations`, and each extension's `rules` are optional, but each one present must be an array.

Rules can be declared in `tool.driver.rules` or in a `tool.extensions[].rules` table, where CodeQL's `--sarif-group-rules-by-pack` places them; a rule ID declared twice anywhere is rejected. A result's `ruleIndex`, and the `index` of its `rule` reference, must be in range of the rule table of the referenced tool component (`rule.toolComponent.index` into `tool.extensions`, otherwise the driver) and name the result's `ruleId`. A `rule.id` must equal `ruleId`, and a `ruleIndex` and `rule.index` that are both present must agree. A tool component referenced only by name or GUID is rejected. `-1` is SARIF's "no index" default and counts as absent.

A location can contain a safe relative URI, a valid index into the run artifact table, or both; when both are present, the URI and the indexed artifact's URI must name the same path. Octet percent-decodes each URI and applies the path policy to the decoded path, which it also stores, so `src/my%20module.rs` becomes `src/my module.rs` and `src/%2e%2e/x.rs` is rejected as traversal. A malformed escape, a decoded path that is not UTF-8 or contains a control character, a colon in its first segment (a drive letter such as `C:/x.rs` or `c:x.rs`, or a scheme such as `file:/x`), or a backslash-separated `..`, and a raw `?` or `#` (a query or fragment) are rejected. A colon in a later segment is part of a file name, so Octet's own SARIF output, which writes `src/a:b.rs` as `src/a%3Ab.rs`, imports back as `src/a:b.rs`. A URI without `uriBaseId`, or with the `%SRCROOT%` base CodeQL emits, is relative to the workspace root; Octet does not read `originalUriBaseIds`, and any other `uriBaseId` is rejected. Each region contains a positive start line and a positive start column; an absent `startColumn` means column 1, the SARIF default that CodeQL omits.

Octet preserves the driver, rule ID, source level, message, path, line, and column. The normalized finding stores source level separately from Octet gate severity.

Octet follows SARIF 2.1.0 result `kind` and `level` defaults. A result without `kind` is a `fail` result. A `fail` result without `level` takes the declared rule's `defaultConfiguration.level`, then `warning`. Octet does not import `pass`, `notApplicable`, or `informational` results and does not gate on them; such a result must omit `level` or set it to `none`. Octet rejects reports with undecided `review` or `open` results and with unknown kinds, because it cannot decide whether they should gate.

Octet rejects malformed JSON, unsupported versions, unknown drivers, missing, `null`, or non-array results and rule tables, unsuccessful tool executions, undeclared or duplicate rules, rule indexes that are out of range or name another rule, unsupported levels or kinds, remote URIs, absolute paths, traversal (including percent-encoded traversal), malformed percent escapes, URI queries and fragments, unsupported `uriBaseId` values, bad artifact indexes, URIs that disagree with their artifact index, missing regions, duplicate normalized findings, and over-limit input.

Genuine CodeQL CLI 2.27.2 output imports under `codeql-local`: the checked fixtures `cargo-octet/tests/fixtures/evidence-rails/sarif-codeql-cli.json` and `sarif-codeql-cli-grouped.json` are trimmed `codeql database analyze --format=sarif-latest` reports for a small Rust crate, without and with `--sarif-group-rules-by-pack`. CodeQL reports columns in UTF-16 code units (`columnKind: utf16CodeUnits`); Octet preserves the reported column number unchanged.

## Artifacts and privacy

The bundle contains the exact raw report, a deterministic normalized projection, a receipt, a summary, and provenance. The projection links to the raw report. Artifact verification remeasures both BLAKE3 identities.

SARIF messages and paths can contain sensitive data. Review the local report before import. The intake does not redact source content and does not contact a network service.

## Non-claims

Imported SARIF findings remain external static observations. They do not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.

Executed producer evidence records that one pinned local scanner ran against one bound source snapshot and retained the report it produced. It does not transfer CodeQL pack, Semgrep, GitHub code scanning, or Semgrep Cloud authority, and a clean result is bounded to the declared rules, scope, exclusions, and unsupported features. The shipped Semgrep and CodeQL producer profiles are declarative pins for real local tool installations; this repository's automated tests exercise the producer shell with a clearly labeled test-only executable fixture instead.
