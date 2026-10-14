# Change: Add Hegel property evidence rail

## Why

Hegel RS is already useful in sibling OnixResearch repos for generated-input properties, and Octet has pure deterministic cores whose evidence contracts benefit from shrinking and reproducible counterexamples. Octet should collect Hegel results as bounded external evidence without making starter checks heavier or allowing property tests to overclaim.

## What Changes

- Add an opt-in Hegel property evidence rail to `cargo octet evidence collect`.
- Add receipt identity, validation, provenance, and Valence export semantics for Hegel property runs.
- Add guidance for using Hegel on pure Octet cores while preserving fixture/UI/snapshot coverage for user-visible behavior.
- Add positive and negative fixtures for passing properties, failing counterexamples, missing identity fields, stale generator hashes, and overclaim attempts.

## Impact

- **Files**: external evidence rail registry, receipt DTO/contracts, artifact verifier, provenance producer registry, Valence export adapter, docs, and fixtures.
- **Consumers**: Valence can ingest Hegel property receipts as bounded generated-testing evidence once the matching Valence receipt contract lands.
- **Non-claims**: Hegel receipts are generated testing evidence only; they are not formal proof, exhaustive coverage, verifier soundness, whole-program correctness, or release eligibility.
