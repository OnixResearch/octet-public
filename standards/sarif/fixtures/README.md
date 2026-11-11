# Real producer fixtures

These fixture crates retain raw SARIF produced by the pinned local producer
profiles, not the test-only scanner executable used by the unit tests.

## Semgrep

- Semgrep `1.180.0`
- Producer profile `semgrep-exec-local`
- Rule bundle `standards/sarif/semgrep-rust-rules.yml`
- Profile bytes `standards/sarif/producers.json`
  (`b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd`)

`semgrep-clean/report.sarif` has zero results. `semgrep-violation/report.sarif`
has one `standards.sarif.octet.rust.expect-without-rationale` error at
`src/lib.rs:5` (Semgrep prefixes the rule ID with the config path). The
retained reports are imported by the checked `semgrep-local` intake profile in
`producer_tests::retained_real_semgrep_reports_import_with_expected_findings`.
Semgrep 1.180.0 emits the driver name `Semgrep OSS`; the importer accepts that
spelling for the checked `Semgrep` profile while preserving older reports.

Reproduce with Semgrep on `PATH` and a workspace layout containing `src/` and
`standards/sarif/semgrep-rust-rules.yml`:

```sh
cargo octet evidence collect \
  --rail sarif-import \
  --profile semgrep-exec-local \
  --config producers.json \
  --workspace-root /path/to/fixture-workspace \
  --run \
  --artifact-dir artifacts/semgrep-violation \
  --output-format json
```

## CodeQL

- CodeQL CLI `2.27.2`
- Query pack `codeql/rust-queries@0.1.46`
- Producer profile `codeql-exec-local`
- Profile bytes `standards/sarif/producers.json`
  (`b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd`)
- CodeQL database build mode `none`

`codeql-clean/report.sarif` has zero results. `codeql-violation/report.sarif`
has one `rust/non-https-url` warning at `src/lib.rs:7` from the reqwest
request-url sink. The retained reports are imported by the checked
`codeql-local` intake profile in
`producer_tests::retained_real_codeql_reports_import_with_expected_findings`.

Reproduce with CodeQL on `PATH` and a copy of the checked producer registry in
the fixture root:

```sh
cp standards/sarif/producers.json standards/sarif/fixtures/codeql-violation/
cargo octet evidence collect \
  --rail sarif-import \
  --profile codeql-exec-local \
  --config producers.json \
  --workspace-root standards/sarif/fixtures/codeql-violation \
  --run \
  --artifact-dir artifacts/codeql-violation \
  --output-format json
```

The producer writes `sarif-raw-report.json`; it is retained here as
`report.sarif`. The fixtures are bounded external static observations. They do
not prove complete vulnerability detection, whole-program correctness, or
release eligibility.
