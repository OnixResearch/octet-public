# Reviewed requirement contracts and freshness

Octet can evaluate a reviewed requirement-to-evidence contract and answer freshness queries against
the same contract. The contract is human-authored Nickel exported to deterministic JSON. The decision
is a Rust-owned DTO that can be recomputed from the retained contract, evidence, and source bytes.

## Commands

```bash
cargo octet requirements check \
  --contract standards/requirements/example-contract.json \
  --evidence standards/requirements/fixtures/example/evidence.json \
  --source-root . \
  --output artifacts/requirements/decision.json \
  --output-format json

cargo octet requirements verify \
  --contract standards/requirements/example-contract.json \
  --evidence standards/requirements/fixtures/example/evidence.json \
  --decision artifacts/requirements/decision.json \
  --source-root .

cargo octet requirements freshness \
  --contract standards/requirements/example-contract.json \
  --index artifacts/evidence-index.json \
  --output-format human
```

Use `cargo octet evidence-index codebase build` to build the index consumed by `freshness`. The
freshness operation is read-only: it never executes a provider.

## Contract shape

A contract declares:

- a stable `contract_id` and a reviewed specification path plus BLAKE3 identity;
- clauses with stable IDs, required/optional status, disposition, checked scope, expected evidence
  tier, allowed evidence IDs, required evidence classes, required negative controls, source bindings,
  and freshness bindings;
- accepted assumptions with an identity, trusted boundary identity, affected clauses, and required
  empirical evidence;
- explicit exclusions with a reason and the reviewed specification identity;
- non-claims.

Dispositions are `formal_proof`, `empirical_check`, `accepted_assumption`, `unresolved`, and
`excluded`. An ID match alone never satisfies a clause.

## Evaluation rules

- Every required clause must have the evidence disposition allowed by the contract and retained
  artifacts whose BLAKE3 identities match the evidence bundle.
- A proof for another requirement, scope, source revision, or specification does not satisfy a
  clause. Proof cards must carry a passing status, `formal_proof` tier, the allowed source
  requirement, the allowed checked scope, explicit non-claims, retained artifact identities, and
  provider evidence when the clause names a required provider.
- Empirical evidence cannot satisfy a formal-proof requirement.
- Required negative controls must use an attributable v2 rejection record with an expected obligation
  and retained verifier evidence. A failed status alone is not a negative control.
- An accepted assumption is bound to its identity and trusted boundary identity. A new assumption or
  widened trusted boundary changes the decision identity and cannot inherit earlier acceptance.
- Missing execution, duplicate clause IDs, an empty required inventory, unresolved required clauses,
  and unsupported dispositions fail closed.
- Exclusions, accepted assumptions, evidence modes, and remaining limitations are visible in human
  and JSON output.

## Freshness rules

Freshness consumes the same contract as its expected-obligation inventory. Each required clause
reports one of:

- `current`: a record satisfies all declared source, policy, tool-identity, artifact, scope, and
  evidence-tier bindings;
- `stale_binding`: related records exist but their direct bindings differ; expected and actual
  identities are shown;
- `missing`: no related record exists and the index declares complete coverage;
- `failed`: a directly bound record reports a failed/rejected/invalid outcome;
- `unsupported`: the matching record family or tier is unsupported;
- `incomplete`: the index, snapshot, or bounded traversal cannot establish complete coverage.

Related records use the existing bounded typed traversal. Shared labels and content identity
establish relatedness only. They never establish causal dependency, invalidate a proof, or supply
proof support.

## Recomputation boundary

`requirements verify` recomputes the decision from the contract, evidence, and source bytes and
compares the recomputed decision identity, clause decisions, assumption decisions, exclusions,
aggregate result, and reviewed specification identity. It does not call an AI model, a hosted
provider, or a signing service.

The contract surface does not authenticate a reviewer or producer. It does not prove verifier
soundness, whole-program correctness, semantic equivalence, or release eligibility. External
producer authenticity belongs to the opt-in artifact-authentication admission path.
