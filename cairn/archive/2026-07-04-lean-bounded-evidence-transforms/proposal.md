## Why

Lean tactics and simplification are powerful because they transform proof goals into smaller obligations, but unbounded automation would be a bad fit for Octet release evidence. Octet should adapt the useful part: named, bounded, auditable transforms that derive or normalize evidence while recording exactly which rules ran and what obligations remain.

This turns tactic-like ideas into deterministic evidence transforms such as policy normalization, proof-card linkage checks, range-proof scaffolding, artifact identity normalization, or claim-boundary rewrites. The transform output must be checkable and fail closed.

## What Changes

- Define bounded evidence transforms as named pure operations with declared input contracts, output contracts, rule sets, budgets, diagnostics, and non-claims.
- Define explicit rewrite/normalization sets for selected policy, receipt, proof-card, and claim-boundary surfaces instead of hidden global simplification.
- Record transform identity and parent artifacts in provenance when transformed evidence is emitted.
- Add positive and negative tests for deterministic transform output, budget exhaustion, unknown rules, malformed input, stale parent artifacts, and overbroad claims.

## Impact

- **Files**: shared proof/artifact identity helpers, policy normalization helpers, proof-card/proof-ref validation, evidence provenance, generated contracts or registry entries, docs for transform boundaries, and fixtures.
- **Testing**: pure transform tests, parent-provenance tests, negative malformed/budget/unknown-rule cases, artifact verifier tests, and Cairn validate/gates.
