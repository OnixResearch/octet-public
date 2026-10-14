## Context

Octet's current surfaces are accurate but scattered. A user can find lint domains, standards registry checks, release smoke transcripts, SARIF output, and semantic fingerprints, but the relationship between those pieces is implicit.

The evidence stack makes that relationship explicit without introducing new runtime behavior. It gives reviewers a stable mental model and gives future output schemas a place to attach evidence-tier information.

## Approach

1. Introduce a `## Evidence stack` section in the README or consumer guide.
2. Define the four layers:
   - discipline: Tiger Style and closed-interface rules;
   - static edge: Dylint lint domains and structural source checks;
   - policy layer: Nickel-authored policies, standards registry, and checker configuration;
   - evidence chain: SARIF, sidecars, transcripts, fingerprints, and lifecycle evidence.
3. For each layer, name what it catches and what it does not claim.
4. Link the evidence-stack model to the separate evidence-tier taxonomy change so output artifacts can later carry machine-readable tier labels.

## Risks

- The stack could imply stronger guarantees than individual checks provide. Mitigate with explicit scope text in each layer and by avoiding the phrase "proof stack."
- The stack could duplicate existing docs. Mitigate by using it as an index with links to detailed docs instead of copying every lint or policy rule.
- Future artifact schemas could drift from docs. Mitigate by making tier taxonomy and provenance changes depend on this model.

## Validation

- Positive: documentation contains all four named layers and maps each to at least one existing Octet surface.
- Negative: documentation validation rejects unqualified `proof stack` language.
- Cairn validation and gates pass for this change.
