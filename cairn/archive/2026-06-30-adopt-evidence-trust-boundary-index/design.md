## Context

The prior changes each cover one adoption theme. Without an index, users may see isolated sections rather than one coherent adoption story. The index should make the map visible while preserving the exact scope of each section.

## Approach

1. Create a concise index section or page titled around Octet's trust boundary or evidence path.
2. Link to the checked-path framing, evidence stack, tier taxonomy, provenance chain, scope-boundary, and controlled-deployment docs.
3. Use one-sentence summaries that name concrete Octet surfaces.
4. Add link validation or a simple deterministic heading inventory.

## Risks

- The index could become another marketing page with unsupported claims. Mitigate by requiring it to link out and by running forbidden-claim checks.
- The index could create dead links while dependent docs are not implemented. Mitigate by allowing links to planned sections only when the corresponding change lands; otherwise link to existing docs.

## Validation

- Positive: all six themes appear in the index and link to a concrete doc section or artifact guide.
- Negative: the index contains no unqualified verified-runtime, proof-stack, or correctness-guarantee claim.
- Cairn validation and gates pass for this change.
