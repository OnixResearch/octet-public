## Context

Slotcar has JSON/SARIF output, artifact directories, review manifests, and fingerprint evidence. These pieces are individually useful but not yet one replayable receipt. A capsule should be a thin composition layer, not a new database.

## Goals / Non-Goals

**Goals:** deterministic capture, deterministic replay, drift diagnostics, and operator-readable receipts.

**Non-Goals:** remote execution, global source database, broad command caching, or correctness proofs.

## Decisions

### 1. Capsule as versioned DTO plus files

**Choice:** Represent a capsule as a versioned DTO that references bounded sidecar files by BLAKE3 hash.

**Rationale:** Large outputs remain inspectable files while the manifest stays stable and diffable.

**Alternative:** One monolithic archive was rejected because it hides line diffs and makes review harder.

### 2. Redacted deterministic environment allowlist

**Choice:** Include only an allowlisted environment summary needed for reproducibility, such as toolchain identity and explicitly configured Slotcar variables.

**Rationale:** Durable evidence must not persist secrets or machine-local noise.

**Alternative:** Capturing all environment variables was rejected as unsafe and nondeterministic.

### 3. Replay status is tri-state

**Choice:** Replay returns `match`, `expected_failure_match`, or `drift` with structured reasons.

**Rationale:** Negative fixtures and known-failure transcripts need first-class acceptance without masking unexpected changes.

## Risks / Trade-offs

**Overclaiming replay** → Diagnostics and docs must say replay proves the same evidence, not program correctness.

**Path instability** → Store repo-relative paths and hashes; reject absolute path-only inputs.
