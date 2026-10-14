## Why

Unison's content-addressed codebase, ability handlers, structured refactors, typed storage, and pure-result caching point to a coherent next roadmap for Slotcar's agent-facing lint workflow. Slotcar already has content identity and evidence concepts in several specs, but the next slice needs one implementation-oriented OpenSpec that ties artifact identity, effect authority, name indirection, session state, durable records, and deterministic caches into reviewable product surfaces.

## What Changes

- Add a content-addressed artifact store contract for prompts, tool schemas, model requests, session blocks, receipts, and skill/tool definitions.
- Add ability-style tool/effect manifests so file, shell, network, secret, browser, scheduler, and remote execution authority is explicit and replayable.
- Add name-to-hash pointer registry behavior for human-friendly names that resolve to immutable artifact definitions.
- Add structured session todo state and typed durable session records for replay, migration, and semantic search.
- Add pure-result cache requirements for deterministic lint/check/doc results keyed by content identity.

## Non-Goals

- Do not copy Unison syntax, UCM workflows, or a language-level algebraic-effect runtime into Slotcar.
- Do not replace Rust, Cargo, Dylint, Git, Nix, or current OpenSpec workflows.
- Do not cache commands that lack deterministic input, environment, and tool-version identity.
- Do not store raw secrets, absolute paths, local usernames, timestamps, or environment dumps in durable identities.

## Capabilities

### New Capabilities

- `artifact-identity-store`: Immutable hash-addressed records for agent/tool/session artifacts.
- `tool-ability-contracts`: Explicit authority manifests and replay handlers for tool effects.
- `name-hash-pointer-registry`: Mutable names that resolve to immutable content definitions.
- `typed-session-storage`: Versioned typed records for sessions, tool calls, outputs, findings, and review state.
- `pure-result-cache`: Deterministic cache keys and receipts for pure check/build/doc outputs.

### Modified Capabilities

- `consumer-integration`: Adds agent-facing replay, identity, pointer, and typed-session contracts.
- `documentation`: Adds structured session todo documentation requirements.
- `performance`: Adds deterministic pure-result cache requirements.

## Impact

- **Files / artifacts**: `openspec/changes/add-unison-inspired-agent-roadmap/**`; future implementation likely touches CLI surfaces, receipt DTOs, cache storage, docs, and tests.
- **Ordering / dependencies**: Artifact identity and normalization should land before name pointers, typed storage, and cache consumers depend on hashes.
- **Verification / evidence**: Future implementation must save deterministic JSON/receipt fixtures for hash stability, pointer rollback, ability denial/replay, session migration, and cache hit/miss behavior.
