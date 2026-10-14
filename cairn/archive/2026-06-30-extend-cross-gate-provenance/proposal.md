## Why

`provenance.jsonl` currently covers `cargo octet check` artifact bundles. Octet's release path also includes standards policy checks, fingerprint review manifests, transcript assertions, generated policy bundles, and downstream rollout evidence. Reviewers still have to connect those gates by convention.

Extending the same provenance model across gates gives release operators one consistent chain from requirement or policy ID to every artifact they inspect.

## What Changes

- Define cross-gate provenance coverage for `octet-standards`, policy validation, fingerprint review manifests, transcript assertions, and generated policy artifacts.
- Add a checker/source registry for known provenance producers and source ID families.
- Support parent artifact references where one gate consumes or summarizes artifacts from another gate.
- Document how release operators inspect a multi-gate provenance chain without treating validation artifacts as formal proofs.

## Impact

- **Files**: evidence/provenance DTOs, standards checker output or receipts, fingerprint review manifest handling, transcript command output, policy registry, release/operator docs, and integration tests.
- **Testing**: positive multi-gate provenance chain plus negative unknown producer, missing parent artifact, hash mismatch, and proof-tier overclaim fixtures.
