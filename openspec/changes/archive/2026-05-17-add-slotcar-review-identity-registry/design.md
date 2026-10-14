## Context

Function fingerprints currently identify source bodies and carry review/evidence metadata. Operators still use names like review classes, evidence aliases, and policy labels; these are useful but mutable. A registry separates ergonomic names from immutable evidence identity.

## Goals / Non-Goals

**Goals:** make review identity durable, make label updates auditable, and preserve rollback for evidence pointers.

**Non-Goals:** global code database, blanket function hashing, or correctness proof from identity matching.

## Decisions

### 1. Registry entries store alias plus immutable target

**Choice:** Each registry entry stores a human alias, target hash, target kind, creation metadata, and optional supersedes pointer.

**Rationale:** This supports rename/rollback while preserving exact evidence identity.

### 2. Review manifests carry both fields

**Choice:** Fingerprint review manifests include the review/evidence alias and the resolved immutable target hash.

**Rationale:** Humans can read names; automation can verify identity.

### 3. Stale labels warn before hard failure

**Choice:** Initial implementation reports stale aliases with deterministic warnings unless a strict profile is selected.

**Rationale:** Existing consumers can adopt the registry incrementally.

## Risks / Trade-offs

**Registry sprawl** → Keep kinds narrow: review class, evidence artifact, policy entry, and fingerprint manifest.

**False proof language** → Diagnostics must say identity/review match, not correctness proof.
