## Context

`concrete_construction_in_core` catches some direct constructors. A function can still expose impurity by returning concrete resource handles from core APIs. Return-type checks make the boundary visible at API design time.

## Decisions

### 1. Check public and private functions in core scopes

**Choice:** Emit for functions in configured core source scopes whose return type contains a known resource or capability handle.

**Rationale:** Even private helpers can spread shell concerns through core modules.

### 2. Use a resource type table

**Choice:** Start with a small table for files, sockets, process/command handles, join handles, channels, and known client/connection types.

**Rationale:** Narrow matching keeps diagnostics explainable and avoids generic type-name policing.

### 3. Allow command values and ports

**Choice:** Domain command enums, serializable plans, and trait-based ports stay clean.

**Rationale:** The desired design is to describe work in core and execute it in the shell.

## Risks / Trade-offs

- Type aliases and wrapper types can hide resources; add coverage only with evidence.
- Some resource handles are domain values in low-level crates. Suppressions should document that boundary decision.

## Validation

Add return-type classifier unit tests and UI fixtures for resource and non-resource API shapes.
