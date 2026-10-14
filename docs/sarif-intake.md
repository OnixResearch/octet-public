# Local SARIF intake

Octet imports a bounded SARIF 2.1.0 subset from an explicit local file. It does not run CodeQL, Semgrep, or a hosted scanner.

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

Use `semgrep-local` for a Semgrep report. Octet rejects `--run` and `--command` for this rail.

The profile binds the driver name, supported version, required rule identity, source-level mapping, local path policy, evidence tier, and resource limits. The checked JSON profile has BLAKE3 identity `b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7`.

## Supported subset

A report contains one local run. The run contains a declared driver, declared rules, bounded results, messages, and one physical location per result. A location can contain a safe relative URI or a valid index into the run artifact table. Each region contains a positive start line and column.

Octet preserves the driver, rule ID, source level, message, path, line, and column. The normalized finding stores source level separately from Octet gate severity.

Octet rejects malformed JSON, unsupported versions, unknown drivers, undeclared or duplicate rules, unsupported levels, remote URIs, absolute paths, traversal, bad indexes, missing regions, duplicate normalized findings, and over-limit input.

## Artifacts and privacy

The bundle contains the exact raw report, a deterministic normalized projection, a receipt, a summary, and provenance. The projection links to the raw report. Artifact verification remeasures both BLAKE3 identities.

SARIF messages and paths can contain sensitive data. Review the local report before import. The intake does not redact source content and does not contact a network service.

## Non-claims

Imported SARIF findings remain external static observations. They do not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.
