## Why

external evidence tooling presents a clear trust stack: formal foundation, static edge, property verification, and audit chain. Octet has analogous but different layers: Tiger Style discipline, static Dylint checks, policy and standards checks, and reviewable evidence artifacts. Those layers exist today, but docs do not present them as one coherent model.

Octet should adopt an evidence-stack model so users understand what each layer contributes and where its boundary sits. The name matters: this is an evidence stack, not a proof stack, because many Octet checks are static heuristics, policy gates, fixtures, transcripts, or review artifacts rather than formal proofs.

## What Changes

- Document the Octet evidence stack with four layers: discipline, static edge, policy layer, and evidence chain.
- Map existing Octet surfaces to each layer: Tiger Style guidance, lint domains, standards registry/checkers, SARIF/sidecars/transcripts/fingerprints/OpenSpec or Cairn evidence.
- Require stack documentation to distinguish evidence, validation, and proof.
- Add validation that consumer-facing docs do not rename the model as a proof stack.

## Impact

- **Files**: `README.md`, `docs/consumer-guide.md`, `docs/lint-reference.md`, `docs/standards/policy-contract.md`, and release/operator docs.
- **Testing**: positive documentation inventory for all four layers; negative text check preventing unscoped "proof stack" language.
