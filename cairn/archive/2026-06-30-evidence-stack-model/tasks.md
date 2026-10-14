## Contract

- [x] [serial] Define the four evidence-stack layers and their scope boundaries. r[octet.evidence_stack.layers]
- [x] [serial] Define the requirement that the model remains an evidence stack, not an unqualified proof stack. r[octet.evidence_stack.no_proof_stack]

## Implementation

- [x] [serial] Add an evidence-stack section to README or consumer guide. r[octet.evidence_stack.layers]
- [x] [serial] Map each evidence-stack layer to concrete Octet surfaces and docs links. r[octet.evidence_stack.layers]
- [x] [serial] Add scope language distinguishing lint evidence, policy validation, transcript assertions, and formal proof where applicable. r[octet.evidence_stack.layers]

## Verification

- [x] [serial] Add or run a positive documentation inventory check that all four layers are present with at least one concrete Octet surface each. r[octet.evidence_stack.layers]
- [x] [serial] Add or run a negative documentation check rejecting unqualified `proof stack` wording. r[octet.evidence_stack.no_proof_stack]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.evidence_stack.layers]
