## Context

The internal corpus pilot added `cargo octet object corpus receipt --output RECEIPT.json [PATHS...]`. The receipt records selected source paths, object count, object-set BLAKE3 hash, caveat distributions, pure-cache blockers, replay metadata, and no-correctness/no-semantic-equivalence boundaries.

## Goals / Non-Goals

**Goals:**
- Recompute the corpus receipt for current source inputs and compare it to a saved receipt.
- Produce deterministic machine-readable check output suitable for local gates.
- Return a non-zero exit code on drift or boundary tampering.

**Non-Goals:**
- Prove behavioral correctness or semantic equivalence.
- Implement HIR/MIR extraction.
- Accept cross-machine determinism as proven by a local check.

## Decisions

### 1. Compare receipt-critical fields instead of raw replay command text

**Choice:** The checker compares schema, object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields.

**Rationale:** These fields define the evidence identity and safety boundary. The saved replay command may name a different output path, so raw full-receipt byte equality would be too brittle for a check command.

**Alternative:** Compare receipt JSON bytes exactly. Rejected because re-checking with a different output path would alter replay metadata without changing evidence identity.

### 2. Fail closed on missing/malformed receipt or boundary drift

**Choice:** Invalid JSON, unsupported schema, or altered no-correctness/no-semantic-equivalence fields produce failed checks.

**Rationale:** A replay gate must not silently accept receipt shape drift or overclaiming boundary changes.

## Risks / Trade-offs

**Receipt field drift** → Keep the comparison explicit and covered by tests so future schema changes require deliberate checker updates.

**Overclaiming** → Include boundary comparison and tests for tampered claims; command help continues to state objects are review/provenance evidence, not correctness proofs.
