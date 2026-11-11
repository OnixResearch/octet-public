## Context

`standards/policies/registry.ncl` already declares `tigerstyle.idioms.nominal-type-boundaries` with a `type-state` mechanism, but the Rust architecture checker currently uses a fixed `RAW_TYPE_STATE_PATTERNS` list. That means generated artifacts do not carry the policy intent, and downstream consumers cannot see or review exactly what a type-state structural gate enforces.

## Goals

- Make type-state structural enforcement artifact-backed like bounded queues and effect capabilities.
- Keep Nickel as the typed authoring surface and Rust as the source-aware checker.
- Start with narrow deterministic obligations that can be fixture-tested without full Rust type solving.

## Non-Goals

- Proving complete protocol correctness.
- Inferring every valid transition from Rust types.
- Broad consumer rollout.

## Decisions

### 1. Type-state obligations are explicit artifact fields

**Choice:** Extend `Mechanism`/Nickel contracts with type-state-specific arrays:
`type_state_source_scopes`, `raw_state_alias_patterns`, `raw_transition_patterns`, `required_state_markers`, and `required_transition_methods`.

**Rationale:** The checker should not infer enforcement intent from prose or hard-coded defaults. Artifact fields make the gate deterministic, reviewable, and portable to downstream CI.

**Alternative:** Continue hard-coded Rust patterns. Rejected because it repeats the pre-artifact weakness fixed for bounded queues and capabilities.

### 2. The first gate rejects raw bypasses and requires visible nominal protocol API evidence

**Choice:** The structural checker fails when declared source scopes contain configured raw state aliases or raw transition functions, and it reports missing required state markers or transition methods when a scoped source root has no visible type-state API evidence.

**Rationale:** This gives a real structural gate with positive and negative fixtures while avoiding overclaiming semantic completeness.

**Alternative:** Require full AST/type analysis immediately. Rejected as higher effort; source-aware Rust analysis can evolve after the artifact seam is real.

### 3. Nominal type boundary policy becomes hard/structural/error for this scoped mechanism

**Choice:** Promote `tigerstyle.idioms.nominal-type-boundaries` to `classification = "hard"`, `enforcement_level = "hard"`, `current_enforcement_mode = "structural"`, and `default_severity = "error"` for declared source scopes.

**Rationale:** A structural gate that exits nonzero on configured raw bypasses must be represented as a hard policy.

## Risks / Trade-offs

- **Pattern precision**: Line-oriented matching can false-positive on comments or examples. Mitigation: ignore comment-only lines and scope the gate to configured source roots.
- **Incomplete protocol proof**: Visible marker names do not prove transition correctness. Mitigation: spec the result as structural API evidence, not verified protocol correctness.
- **Policy churn**: Promoting a policy to hard requires fixtures and evidence. Mitigation: add compliant/noncompliant fixtures and saved transcripts before archive.
