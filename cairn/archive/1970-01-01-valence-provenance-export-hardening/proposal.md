# Change: Valence provenance export hardening

## Why

Octet is the checked Rust path in the stack. Its provenance export should be
adapter-ready without letting lint, review, imported proof, or validation
artifacts overclaim. Valence needs stable Octet receipt identities, typed edge
semantics, and preserved evidence tiers before it can bind Octet facts to Mantle
release bundles.

## What Changes

- Harden Octet Valence provenance exports so every exported record is backed by
  a replayable Octet receipt envelope with tool identity, normalized invocation,
  policy/checker version, input artifact IDs, output artifact IDs, status, and
  failure reason when applicable.
- Emit typed provenance edges that distinguish checks, proof receipts, imported
  Trellis references, parent artifacts, and Valence adapter inputs.
- Preserve evidence tiers so imported Trellis artifacts remain manual-review or
  reference inputs unless Octet reruns and validates an Octet-owned proof receipt.
- Add positive and negative fixtures for valid exports, missing parents, stale
  digests, unknown producers, unsupported edge kinds, and manual-review evidence
  promoted to formal proof.

## Impact

- **Files**: Octet provenance export core, producer registry/policy, fixtures,
  CLI docs, and artifact verification tests.
- **Consumers**: Valence receives deterministic, typed, fail-closed inputs.
- **Non-claims**: Octet exports verified artifact facts and scoped proof receipts;
  the export does not prove Mantle bundle inclusion, release eligibility,
  whole-program correctness, or verifier soundness.
