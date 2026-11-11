# Prove one candidate: walkthrough transcript

This transcript walks the `docs/skills/prove-a-candidate/SKILL.md` path on the
fixture crate in `fixtures/proof-candidates/`. It uses the existing
`octet.verus_proof_rails.addition-bound-production#rail-0` card because that is
the rail the README narrowing example links. The rail proves the declared
addition-bound property only; the negative check below records
`property_not_demonstrated` for the narrowed codec source.

Commands that run `proof-ref scaffold` and `proof-ref validate` execute from
`fixtures/proof-candidates` so `src/codec.rs` is the consumer source path.
Candidate listing runs before the reference from the repository root and after
the reference from the fixture root. Artifacts live under a temporary directory;
the recorded card deck is checked in at
`fixtures/proof-candidates/evidence/proof-cards.json`.

```octet-transcript
$ cargo octet prove --candidates --source fixtures/proof-candidates/src --trust-boundary fixtures/proof-candidates/trust-boundary.md --json
=> schema_version=octet-proof-candidates/v1
=> candidate lint=unchecked_narrowing path=fixtures/proof-candidates/src/codec.rs line=42 rank_score=5 trust_boundary=true
=> proof_ref status=missing
=> rail proof_or_property_id=octet.verus_proof_rails.addition-bound-production#rail-0 unsupported=false

$ mkdir -p "$ART/policy" && nickel export --format json standards/policies/registry.ncl > "$ART/policy/octet-policies.json"
=> policy registry export written to $ART/policy/octet-policies.json

$ cargo octet prove --explain --policy-artifacts-dir "$ART/policy" --source src/verified --rail 'octet.verus_proof_rails.addition-bound-production#rail-0'
=> octet prove status=passed proofs=1 diagnostics=0
=> proof policy_id=octet.verus_proof_rails.addition-bound-production status=passed checked_scope=formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0

$ cargo octet prove --policy-artifacts-dir "$ART/policy" --source src/verified --rail 'octet.verus_proof_rails.addition-bound-production#rail-0' --artifact-dir "$ART/proofs"
=> octet prove artifact_dir=$ART/proofs status=passed proofs=1 diagnostics=0
=> proof status=passed receipt=proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt

$ (cd fixtures/proof-candidates && cargo octet proof-ref scaffold --proof-cards "$ART/proofs/proof-cards.json" --proof-or-property-id 'octet.verus_proof_rails.addition-bound-production#rail-0' --reference-id unchecked-narrowing-range-proof --consumer-context 'allow tigerstyle::unchecked_narrowing at src/codec.rs:42' --consumer-source src/codec.rs --anchor 'octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0' --output proof-ref.json)
=> proof-ref.json written with schema_version=octet-proof-evidence-ref/v1

$ (cd fixtures/proof-candidates && cargo octet proof-ref validate --proof-cards "$ART/proofs/proof-cards.json" --ref proof-ref.json)
=> octet proof-ref reference_id=unchecked-narrowing-range-proof valid=true matched_card=true consumer_source_matched=true diagnostics=0

$ cargo octet artifact verify --artifact-dir "$ART/proofs"
=> octet artifact verification status=valid
=> artifact=proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt evidence_tier=formal_proof

$ (cd fixtures/proof-candidates && cargo octet prove --candidates --proof-cards "$ART/proofs/proof-cards.json" --trust-boundary trust-boundary.md --json)
=> schema_version=octet-proof-candidates/v1
=> candidates=0 discharged=1 diagnostics=0

$ cargo octet prove --policy-artifacts-dir "$ART/policy" --source "$MUTANT/src/verified" --rail 'octet.verus_proof_rails.addition-bound-production#rail-0' --artifact-dir "$ART/mutant-proofs"
=> octet prove artifact_dir=$ART/mutant-proofs status=passed proofs=1 diagnostics=0
=> proof status=passed

$ cargo octet prove --record-negative-check --proof-cards "$ART/proofs/proof-cards.json" --mutant-cards "$ART/mutant-proofs/proof-cards.json" --proof-or-property-id 'octet.verus_proof_rails.addition-bound-production#rail-0' --mutant-source src/verified/addition.rs --output fixtures/proof-candidates/evidence/proof-cards.json
=> octet prove negative-check status=property_not_demonstrated
=> diagnostic: rail `octet.verus_proof_rails.addition-bound-production#rail-0` still passes against the mutant source `src/verified/addition.rs`: property not demonstrated
=> the recorded `proof-cards.json` negative_check status is property_not_demonstrated
```
