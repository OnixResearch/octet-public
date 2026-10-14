## Context

Existing Slotcar specs already cover named capabilities and effect-surface reporting. This change narrows that into a policy contract: every effect-like lint family needs a vocabulary mapping or an explicit unknown status.

## Goals / Non-Goals

**Goals:** make ambient effects visible, keep core/shell boundaries reviewable, and prevent new lints from bypassing effect summaries.

**Non-Goals:** source-level annotations everywhere, algebraic-effect runtime, or default-deny gating for advisory families without fixtures.

## Decisions

### 1. Vocabulary first, enforcement second

**Choice:** Start with stable names and output mapping, then promote hard gates only when fixtures prove low false-positive risk.

**Rationale:** Capability language helps review immediately without causing broad false-positive cleanup waves.

### 2. Unknown capability is explicit

**Choice:** A lint with no capability mapping must emit `unknown` or fail policy validation depending on release profile.

**Rationale:** Silent omission makes summaries misleading.

### 3. FCIS is the review frame

**Choice:** The vocabulary is documented in terms of deterministic functional core versus imperative shell authority.

**Rationale:** This maps Unison-style abilities to Slotcar's existing Rust standards model.

## Risks / Trade-offs

**Noisy early output** → Keep new families advisory until fixture-backed promotion.

**Vocabulary drift** → Validate policy registry entries against a single typed capability list.
