## Context

Octet already requires functional-core / imperative-shell decomposition. This change makes the effect side of that contract concrete for runner and standards code: shell code captures or performs effects; pure cores receive typed facts, command results, artifact bytes, policy values, or narrow capabilities.

## Decisions

### 1. Core functions receive data, not ambient effects

**Choice:** Pure cores SHOULD not read environment variables, current directories, filesystem paths, wall-clock time, randomness, process output, logs, metrics, network state, async runtime state, or global mutable state directly.

**Rationale:** Hidden inputs make deterministic tests and evidence receipts fragile.

### 2. Shells own acquisition and execution

**Choice:** CLI shells and integration shells SHOULD own file traversal, process execution, environment allowlists, working directories, output writing, logging, metrics, timeout enforcement, and random or clock values.

**Rationale:** This keeps side effects visible and lets pure cores operate over in-memory values.

### 3. Use narrow capabilities only when values are insufficient

**Choice:** Prefer passing concrete values into pure cores. When a behavior truly needs deferred access, use a small trait or capability token for one effect family rather than a broad service object.

**Rationale:** Small capabilities are reviewable and law-testable. Broad traits recreate ambient state behind a mock.

### 4. Return structured events or plans

**Choice:** Pure cores SHOULD return decisions, diagnostics, write plans, command plans, or structured observability events instead of printing, logging, spawning, or writing directly.

**Rationale:** The shell can enact the plan and record evidence while tests assert the pure output.

### 5. Align with lint and manifest rails

**Choice:** Existing purity-effect lints and capability manifests SHOULD be used as guardrails where available, but this change remains an architecture contract rather than a claim that Octet implements a full Rust effect system.

**Rationale:** Claim boundaries matter: source-shape checks and explicit seams support review, but they do not prove whole-program purity.

## Risks / Trade-offs

- Over-injection can make code harder to read; prefer concrete input values over traits when possible.
- Shell/core splits can be noisy if done mechanically; target high-value seams first.
- Tests should not become mock-driven for pure logic; fake capabilities belong at shell seams.
