## Context

Slotcar is a Rust lint and agent-facing guidance project. Prior Unison-inspired changes added focused content identity, effect surface, executable documentation, migration, and cache ideas. This roadmap change collects the next product-facing layer: immutable artifact identities, explicit authority manifests, mutable names over immutable hashes, typed session records, structured work state, and deterministic pure-result caches.

## Goals / Non-Goals

**Goals:**
- Make artifact identity a reusable substrate for replay, audit, cache keys, and pointer rollback.
- Make tool authority explicit enough to enforce, simulate, deny, and replay.
- Persist session records in stable typed DTOs without leaking local or secret data.
- Keep the implementation Rust-first, OpenSpec-backed, and locally verifiable.

**Non-Goals:**
- No Unison runtime, syntax, code database, or Cloud API compatibility.
- No blanket caching for non-deterministic commands.
- No raw secret or local-machine metadata persistence.

## Decisions

### 1. Normalize artifacts before hashing

**Choice:** The artifact store hashes normalized DTOs for prompts, tool schemas, model requests, session blocks, receipts, and skill/tool definitions, using explicit schema version and `b3:` BLAKE3 identity.

**Covers:** Scenario: Hash normalized artifacts; Scenario: Inspect and replay by hash

**Rationale:** A stable normalized input prevents whitespace, path, timestamp, or local-environment drift from defining identity.

**Alternative considered:** Hash raw text blobs only. Rejected because raw blobs are easy to poison with unstable local data and are weak replay contracts.

**Implementation:** Define Rust-owned DTOs, derive deterministic serialization, store by digest, and add inspect/replay fixtures.

**Boundary cases:** Hash input must omit secrets, absolute paths, timestamps, usernames, and environment dumps; replay must fail closed when referenced definitions are missing.

### 2. Treat tool effects as ability-style manifests

**Choice:** Tool calls declare required effects such as `FileRead`, `FileWrite`, `Shell`, `Network`, `Secret`, `Browser`, `Scheduler`, and `RemoteExec`; handlers enforce policy and can record or replay traces.

**Covers:** Scenario: Manifest gates authority; Scenario: Replay handler denies unexpected effects

**Rationale:** This imports Unison's ability discipline without language-level effects, giving review gates and subagents a concrete authority vocabulary.

**Alternative considered:** Continue inferring authority from tool names and prose. Rejected because it is not deterministic enough for replay or denial tests.

**Implementation:** Add a manifest DTO, policy checker, negative fixtures for denied effects, and replay handler receipts.

**Boundary cases:** Existing tools may need compatibility defaults, but new deterministic receipts must name the effective authority set.

### 3. Separate human names from immutable definitions

**Choice:** Names are mutable pointers to immutable artifact hashes; pointer updates produce receipts containing old hash, new hash, actor/context, and rollback hint.

**Covers:** Scenario: Name resolves to immutable hash; Scenario: Pointer update supports rollback

**Rationale:** Human-friendly names remain usable while reviews, caches, and replays depend on immutable identity.

**Alternative considered:** Make names part of artifact identity. Rejected because renames and rollbacks would obscure whether the executable definition changed.

**Implementation:** Add pointer registry storage, inspect output, rollback tests, and name-resolution audit receipts.

**Boundary cases:** A deleted or missing target hash must not silently resolve to a newer definition.

### 4. Store session state as typed records

**Choice:** Persist session blocks, model requests, tool calls, tool outputs, findings, todo state, and review decisions as versioned typed records with migrations.

**Covers:** Scenario: Session records are typed and versioned; Scenario: Migration preserves replay identity; Scenario: Session todo state is structured; Scenario: Resolved and unresolved work remain distinguishable

**Rationale:** Typed records enable search, migration, replay, and safe summaries better than unstructured JSONL text alone.

**Alternative considered:** Keep only append-only transcript text. Rejected because it makes semantic search and migration brittle.

**Implementation:** Introduce DTOs plus migration fixtures; document structured todo states for pending edits, failed checks, unresolved findings, and stale specs.

**Boundary cases:** Raw logs can remain attachments, but durable records must carry redacted handles and schema versions.

### 5. Cache only deterministic pure results

**Choice:** Cache check/build/doc results only when command, input hashes, config hash, tool revision, and environment allowlist form a complete key.

**Covers:** Scenario: Pure cache hit reuses matching result; Scenario: Environment drift invalidates cache

**Rationale:** This adapts Unison's no-build intuition to Rust/Nix without pretending impure commands are safe to reuse.

**Alternative considered:** Cache by command line and file mtimes. Rejected because it misses config/tool/env identity and produces weak receipts.

**Implementation:** Add cache-key DTOs, hit/miss receipts, positive hit fixtures, and negative invalidation fixtures.

**Boundary cases:** Network, secret, clock, random, and undeclared filesystem effects must disable or partition cache entries.

## Deterministic Proof Plan

- scenario or requirement:
  - Scenario: Hash normalized artifacts
  - Scenario: Inspect and replay by hash
  - Scenario: Manifest gates authority
  - Scenario: Replay handler denies unexpected effects
  - Scenario: Name resolves to immutable hash
  - Scenario: Pointer update supports rollback
  - Scenario: Session records are typed and versioned
  - Scenario: Migration preserves replay identity
  - Scenario: Session todo state is structured
  - Scenario: Resolved and unresolved work remain distinguishable
  - Scenario: Pure cache hit reuses matching result
  - Scenario: Environment drift invalidates cache
- artifact path:
  - `openspec/changes/add-unison-inspired-agent-roadmap/evidence/*.txt`
- command / helper:
  - future deterministic fixture helpers for artifact hashing, ability denial, pointer rollback, record migration, todo extraction, and cache hit/miss behavior
- exact expected output / acceptance signal:
  - each artifact prints the scenario name, the input fixture path, and a PASS line for the expected hash/denial/rollback/migration/cache condition
- ordering / prerequisites:
  - artifact DTO normalization precedes pointer, typed storage, and cache implementation
- cleanup / postcondition:
  - fixtures leave no untracked cache blobs or local machine paths outside the evidence directory

## Risks / Trade-offs

- Scope creep across several product surfaces -> keep this change as roadmap/spec only until each implementation slice is small and fixture-backed.
- False determinism claims -> require receipts to name omitted/allowlisted environment and reject undeclared effects.
- Secret leakage in durable records -> store redacted handles and hashes only, never raw secret values.
