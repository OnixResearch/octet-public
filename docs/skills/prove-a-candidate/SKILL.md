---
name: prove-a-candidate
description: Walk one Tiger Style lint allowance from an unproven candidate to a validated proof-ref, then run a mutant negative check before claiming the proof is about the intended property. Use in the Octet repo when `cargo octet prove --candidates` reports an allowance without a validated proof-ref.
compatibility: Octet repository checkout with `cargo octet`, an existing `standards/proof-rails/manifest.ncl` rail, and (for production rails) the pinned `octet-production-verus` binary.
---

# Prove a candidate

This skill takes exactly one allowance from `cargo octet prove --candidates` to
a linked proof card and a recorded negative check. It does not invent a proof,
does not claim that an existing rail proves a new property, and does not promote
linkage evidence to whole-program correctness.

## Claim boundary

- A `proof-ref` is linkage evidence. It binds a source site to a saved proof
  card; it does not rerun the verifier and does not emit `formal_proof`.
- A rail proves only its declared `checked_scope`.
- The negative check is the only step that tests whether the rail is about the
  intended property. A mutant that still passes the rail must be recorded as
  `property_not_demonstrated`.
- A mutant failure is evidence for that mutation only. It is not a general
  theorem-to-code equivalence claim.
- When no existing rail matches the lint, report the candidate as unsupported.
  Do not invent a rail mapping.

## Steps

### 1. List candidates

Run from the repository root or the consumer package root:

```bash
cargo octet prove --candidates \
  --source src \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --json
```

Pick one candidate. Record its `path`, `line`, `lint`, and mapped rail (if any).
The candidate stays listed until a validated proof-ref binds that exact
`path:line`.

### 2. Decide whether an existing rail can be linked

- If `rail` is `null`, stop. Write down `unsupported lint=<lint>` and do not
  claim a proof path.
- If `rail` exists, read its `scope_note`. An existing rail may prove a different
  property. Linkage is still allowed, but the negative check must decide whether
  the rail is about the intended property.
- To add a real rail, scaffold a new Verus target beside the existing proofs
  under `verus/proofs/` and add a typed entry to
  `standards/proof-rails/manifest.ncl`. Do not reuse a rail's `source_paths`,
  `proof_paths`, `pass_signal`, verifier identity, or `checked_scope` for a
  different property. Use the existing rail entries as the shape reference.

### 3. Run the rail in explain mode, then for real

```bash
cargo octet prove --explain \
  --policy-artifacts-dir artifacts/policy \
  --source src/verified \
  --rail 'octet.verus_proof_rails.addition-bound-production#rail-0'

cargo octet prove \
  --policy-artifacts-dir artifacts/policy \
  --source src/verified \
  --rail 'octet.verus_proof_rails.addition-bound-production#rail-0' \
  --artifact-dir artifacts/octet-proofs
```

Accept only a `status=passed` proof row and a validated receipt. `--explain`
adds remediation for stale bindings and verifier mismatches. Do not continue on
`manual_review_required`, `failed`, or a missing receipt.

Generate the policy artifact from the registry when it is not already present:

```bash
mkdir -p artifacts/policy
nickel export --format json standards/policies/registry.ncl > artifacts/policy/octet-policies.json
```

### 4. Write and validate the proof-ref

Place the anchor comment next to the allowance site and scaffold the reference
from the saved card:

```bash
cargo octet proof-ref scaffold \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --proof-or-property-id 'octet.verus_proof_rails.addition-bound-production#rail-0' \
  --reference-id unchecked-narrowing-range-proof \
  --consumer-context 'allow tigerstyle::unchecked_narrowing at src/codec.rs:42' \
  --consumer-source src/codec.rs \
  --anchor 'octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0' \
  --output proof-ref.json

cargo octet proof-ref validate \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --ref proof-ref.json
```

Validation must report `valid=true` and `consumer_source_matched=true`. The
candidate should now drop off `prove --candidates` when the same `--proof-ref`
is passed.

### 5. Verify the artifact bundle

```bash
cargo octet artifact verify --artifact-dir artifacts/octet-proofs
```

The bundle must report `status=valid`. Artifact verification checks saved bytes
and provenance; it is not whole-program proof.

### 6. Run the negative check

Create a mutant variant of the function or proof target with the intended guard
or bound removed. Run the same rail into a separate artifact directory:

```bash
cargo octet prove \
  --policy-artifacts-dir artifacts/policy \
  --source <mutant-root>/src/verified \
  --rail 'octet.verus_proof_rails.addition-bound-production#rail-0' \
  --artifact-dir artifacts/octet-proofs-mutant
```

Record the comparison in the baseline card deck:

```bash
cargo octet prove --record-negative-check \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --mutant-cards artifacts/octet-proofs-mutant/proof-cards.json \
  --proof-or-property-id 'octet.verus_proof_rails.addition-bound-production#rail-0' \
  --mutant-source <mutant-source-path> \
  --verifier-evidence artifacts/octet-proofs-mutant/verifier-evidence.json
```

The verifier evidence is a typed `octet-proof-negative-check-evidence/v1` document
that binds the approved specification, obligation, checked scope, provider,
toolchain, verification configuration, mutation patch, and measured baseline and
mutant source/deck bytes. It records the baseline as accepted and the mutant as
rejected for the named obligation, with optional retained raw output. A positive
result without this evidence is inconclusive; an annotated `failed` card status
alone is not a negative control.

- `property_demonstrated`: the mutant was rejected for the named obligation.
  State that this is evidence for this mutation only.
- `property_not_demonstrated`: the rail still passed. Do not claim the intended
  property is proven. Report the exact diagnostic and leave the candidate's
  underlying claim unproven.
- `inconclusive`: the mutant run did not produce an attributable rejection.
  Fix the evidence or stop.

The result is stored in the proof card under `negative_check` in
`proof-cards.json`; `proof-ref validate` recomputes it from the retained
artifacts.

### 7. Report

State:

- candidate `lint` and `path:line`;
- mapped proof/property ID and its `checked_scope`;
- proof-ref validation result;
- artifact verification result;
- negative-check `status` and whether the intended property was demonstrated;
- explicit non-claims from the card and the skill boundary above.

Never describe a passing rail, a validated proof-ref, or artifact verification as
proof of the narrowing, overflow, or indexing property unless the negative check
also reports `property_demonstrated` for a mutation that targets that property.
