## Context

This fixture models tasks-stage obligation checking.

## Goals / Non-Goals

**Goals:** prove tasks gate behavior for scenario-specific verification obligations.
**Non-Goals:** runtime changes.

## Decisions

### 1. Scenario-specific verification obligations

**Choice:** Require exact tasks for absent defaults and explicit warn behavior.
**Covers:** Task obligations map scenarios to exact proof; Task obligation fixtures prove blocked and covered tasks.
**Obligation cross-check:** Verification tasks must name exact scenario, command/helper, input, evidence, and acceptance signal.
**Rationale:** Broad verification tasks can miss required behavior.
**Boundary cases:** Shared evidence is allowed only with separate scenario rows and exact acceptance.
**Proof plan:** tasks gate fixture runner checks blocked/pass behavior.
