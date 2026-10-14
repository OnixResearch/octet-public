## Context

`cargo octet check --artifact-dir` writes `command.txt`, `status.json`, `summary.txt`, optional structured findings, and `provenance.jsonl`. Consumer integration tests already verify this during runner tests, but operators need a reusable command that can verify a saved bundle later.

The verifier should keep deterministic validation logic in a pure core. The shell may read artifact files, compute BLAKE3 hashes, parse JSONL, and render reports.

## Approach

1. Define a bundle verification input containing the expected output format, artifact names, artifact bytes or hashes, and parsed provenance records.
2. Implement a pure validation core that returns deterministic diagnostics and summary rows without reading the filesystem.
3. Add an imperative CLI shell, likely `cargo octet artifact verify --artifact-dir <dir>`, that loads files, rejects stale extras, computes BLAKE3, parses provenance JSONL, and calls the core.
4. Render stable JSON and human summaries that list each artifact, source ID, checker ID, evidence tier, BLAKE3 hash, checked scope, and manual action when applicable.
5. Document the verifier as the local release gate for artifact directories.

## Risks

- Output-format inference can be ambiguous if a bundle is partial. Mitigate by requiring an explicit `--output-format` option or deriving only after the file set is complete.
- The verifier could duplicate test-only helper logic. Mitigate by moving expected-file and provenance validation rules into production core functions.
- Human summaries may tempt overclaims. Mitigate by preserving `manual_review_required` wording and scope boundaries in every summary.

## Validation

- Positive: a complete human, JSONL, and SARIF artifact bundle verifies cleanly.
- Negative: missing required artifact, stale extra file, malformed provenance line, BLAKE3 mismatch, unknown checker ID, invalid evidence tier, and missing manual-review action all fail closed.
- Cairn validation and gates pass for this change.
