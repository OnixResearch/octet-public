# Kani property checklist

Use this checklist when a module adopts Kani or property-test evidence. Select only classes that fit the module and record skipped classes with a short rationale.

## Property classes

| Label | Use for | Positive evidence | Negative evidence |
| --- | --- | --- | --- |
| `normalization` | constructors or repair functions that establish a canonical invariant | normalized output and idempotence | malformed state, boundary values, stale receipt |
| `algebraic-law` | operations with identities, inverses, associativity, or commutativity | law holds over admitted inputs | invalid domain, overflow boundary, non-law fixture |
| `encode-decode` | parsers, serializers, split/join APIs | round trip or decode(encode(x)) | malformed bytes, truncated input, stale schema |
| `validity-range` | predicates for dates, units, sizes, or indexes | accepted range boundaries | underflow, overflow, out-of-range input |
| `loop-termination` | loops with non-obvious progress | invariant plus decreases or complete unwind | missing decreases, failed unwind, non-progress fixture |
| `trait-consistency` | `Eq`, `Ord`, `Hash`, or display/parse contracts | equality agrees with ordering or hashing where required | inconsistent comparison or hash fixture |
| `boundary-panic` | unsafe, FFI-adjacent, indexing, unwrap, and arithmetic boundaries | panic-freedom over admitted inputs | invalid assumption, overflow, out-of-bounds input |

## Evidence rules

- Pair every adopted class with positive and negative evidence.
- Prefer Kani contracts or passing unwind coverage when the claim is stronger than panic-freedom.
- Treat `kani::assume` as proof-critical review input; vacuous assumptions invalidate the harness.
- Plain Kani stubs stay boundary assumptions unless linked to a passing verified contract receipt.
- Kani receipts remain scoped model-check evidence, not whole-program or verifier-soundness proof.

## Receipt label

Kani rail summaries may label properties with `property_class`. Unknown labels fail receipt validation so evidence stays policy-owned and searchable.
