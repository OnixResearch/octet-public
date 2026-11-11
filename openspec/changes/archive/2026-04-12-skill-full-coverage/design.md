# Design: Expand Tiger Style Skill Coverage

## Context

The Tiger Style reference document is the source of truth. The skill is the agent-facing condensed form. Today the skill is useful but incomplete: it keeps core themes while omitting many concrete rules and patterns that drive correct application in practice.

This change is about **coverage fidelity**, not changing Tiger Style itself. The skill should remain skimmable, but it must not silently omit major normative guidance.

## Goals / Non-Goals

**Goals:**

- Make the skill complete enough that an agent can apply Tiger Style without separately rediscovering missing rules
- Preserve fast scanning by grouping related rules instead of pasting the full reference verbatim
- Add explicit coverage for performance and documentation guidance now missing from the spec taxonomy
- Make omissions detectable during future edits

**Non-Goals:**

- Rewriting the reference document
- Adding automated parsing of markdown semantics in this change
- Changing the Rust lint implementation scope directly

## Decisions

### 1. Treat the reference as normative, skill as compressed-but-complete

**Choice:** The skill will remain shorter than the reference, but every major normative rule from the reference MUST appear either:

1. directly in the skill text, or
2. in an explicit grouped checklist with a named pointer to the reference section for examples.

**Rationale:** The skill exists for fast operational use. Full verbatim duplication would make it harder to scan. Silent omission is worse than repetition, so grouped checklists are acceptable when examples live in the reference.

### 2. Add missing domains to the spec taxonomy

**Choice:** Introduce `performance` and `documentation` spec domains.

**Rationale:** Current domains cover safety, naming, structure, purity, and concurrency, but the known gaps include performance guidance and skill-maintenance/documentation requirements. Without dedicated domains, these rules have no stable home.

### 3. Group missing guidance by application area

**Choice:** Fill gaps using grouped requirements:

- safety: assertion patterns, error-path rigor, compiler strictness, explicit defaults
- performance: bandwidth/latency, frequency-weighted optimization, predictability, compiler-independence, worked napkin math
- naming: behavioral naming, name overloading, options structs, comment intent
- structure: signatures, object construction, references, dimensionality, formatting, documentation norms
- purity/verus: two-file pattern, proof patterns, commands, best practices
- concurrency: `unsafe` quarantine in addition to existing lock-order guidance

**Rationale:** Grouping matches how agents apply the skill during coding and review.

### 4. Require an explicit coverage crosswalk

**Choice:** Implementation should include a maintained coverage checklist or crosswalk from reference sections to skill sections.

**Rationale:** This change fixes a drift problem. Without a crosswalk, future edits can regress back to partial coverage.

## Risks / Trade-offs

**[Skill bloat]** Adding every missing rule could make the skill too long.  
**Mitigation:** Keep examples minimal in the skill; push detailed examples to the reference and use grouped bullets.

**[Taxonomy sprawl]** New spec domains increase openspec surface area.  
**Mitigation:** Add only domains with stable conceptual boundaries: `performance` and `documentation`.

**[False completeness]** A broad summary can still miss subtle guidance.  
**Mitigation:** Require section-by-section crosswalk validation before closing the change.
