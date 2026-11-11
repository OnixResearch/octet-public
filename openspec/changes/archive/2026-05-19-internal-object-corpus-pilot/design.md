## Context

The archived content-addressed Rust evidence change added `function-object-v1`, bundle export, pointer validation, real-code corpus tests, and strong local determinism receipts. Those rails currently rely on test transcripts and a three-file corpus. Before downstream pilots, Octet needs a self-contained receipt command over a broader internal corpus.

## Goals / Non-Goals

**Goals:**
- Add a deterministic JSON receipt for object corpus pilots.
- Exercise a broader Octet implementation corpus locally.
- Keep caveats and no-correctness boundaries explicit.

**Non-Goals:**
- No Unison-style callable CAF runtime.
- No rustc/HIR/MIR semantic hashing.
- No downstream consumer pilot.
- No pure-cache eligibility promotion.

## Decisions

### 1. Add `object corpus receipt`

**Choice:** Implement `cargo octet object corpus receipt --output RECEIPT.json [PATHS...]`.

**Rationale:** A command-level JSON receipt is easier to replay and consume than a test-only transcript while reusing the existing function-object scanner and caveat model.

**Alternative:** Keep the receipt as a test helper only. Rejected because it would not expose a stable pilot surface to future workflow automation.

**Implementation:** Reuse `collect_objects`, summarize objects deterministically, sort source paths and caveat counters, compute a BLAKE3 object-set hash from sorted object hashes, and write pretty JSON to the requested path.

### 2. Keep receipt claims narrow

**Choice:** Include explicit boundary fields in the receipt instead of silently implying readiness.

**Rationale:** The v1 scanner is source-token based. The receipt must be useful evidence without overclaiming dependency resolution, macro expansion, purity, semantic equivalence, or correctness.

## Risks / Trade-offs

**Receipt churn** → Minimize by deriving fields from deterministic object records and sorted maps.

**Overclaiming** → Mitigate with required no-correctness boundary assertions in tests and evidence.
