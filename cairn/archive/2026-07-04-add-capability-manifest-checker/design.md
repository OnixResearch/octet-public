## Context

Octet already flags many direct effects through purity lints and documents approved capability families. The missing step is a manifest that lets a repository say which scopes are core, shell, adapter, test, or remote, which capabilities are authorized there, and which deterministic handlers prove testability. This adapts Unison's ability visibility into Rust-compatible policy and evidence rather than language-level syntax.

## Decisions

### 1. Nickel owns human-authored capability manifests

**Choice:** The capability manifest mechanism SHOULD be Nickel-authored with typed contracts and exported to deterministic runtime JSON.

**Rationale:** Consumers will compose and override scope/capability policy. Nickel catches malformed manifests before the checker consumes them and matches Octet's config ownership rules.

### 2. Scope classes are explicit

**Choice:** A manifest MUST classify paths or crates into recognized scope classes such as core, shell, adapter, test, generated, and remote-execution. Unknown scope classes fail closed unless marked as an advisory extension.

**Rationale:** Capability policy only has meaning when authority is tied to scope. Hidden defaults risk silently skipping core obligations.

### 3. Capability families stay vocabulary-owned, not ad hoc strings

**Choice:** Capability family names MUST come from Octet's approved vocabulary, including time, randomness, filesystem, network, process, environment, secrets, remote-execution, unsafe-code, global-state, concurrency, logging/observability where supported, and unknown for classified gaps.

**Rationale:** Stable names make reports queryable and prevent every consumer from inventing incompatible effect taxonomies.

### 4. Checker reuses observations before adding broad analysis

**Choice:** The first implementation SHOULD reuse existing lint observations, source-shape effect classifications, and policy scopes before adding deeper HIR/MIR or type-flow extraction.

**Rationale:** This keeps the first rail reviewable and avoids overpromising transitive effect inference.

### 5. Deterministic handlers are evidence, not mocks hidden in tests

**Choice:** For safety-relevant capabilities, reports SHOULD name deterministic handler fixtures or scripted responses that exercise success and failure paths.

**Rationale:** The goal is testable functional cores with explicit interpreters. Handler evidence proves the boundary is exercised without standing up the world.

### 6. No effect-system overclaim

**Choice:** Documentation and reports MUST state that the checker validates declared source-shape and policy obligations only. It does not add Rust effect types, prove transitive purity, or prove absence of all runtime effects.

**Rationale:** Octet can enforce useful boundaries while staying honest about analysis limits.

## Risks / Trade-offs

- Existing consumers may not have clean core/shell splits. Keep strict diagnostics opt-in and provide an advisory report mode.
- Trait-based capability injection can be hard to distinguish from generic abstractions. Start with declared trait/token names and explicit policy bindings.
- The checker may duplicate some lint findings. Reports should cross-link existing lint IDs instead of emitting disconnected evidence.
