## Phase 1: Implementation

- [x] I1 Implement normalized artifact DTOs and content-addressed storage for prompt, tool-schema, model-request, session-block, receipt, and skill/tool-definition records ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] I2 Implement tool ability manifests and policy/replay handlers for declared effect families ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] I3 Implement name-to-hash pointer registry updates, inspect output, and rollback operation ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] I4 Implement typed durable session record DTOs and migration handling ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] I5 Document structured session todo states and add machine-readable display or export path ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] I6 Implement pure-result cache key DTOs, cache hit/miss receipts, and eligibility checks ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Hash normalized artifacts" with artifact hash fixtures; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/artifact-hash-normalization.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V2 Verify scenario "Inspect and replay by hash" with inspect/replay fixtures; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/artifact-inspect-replay.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V3 Verify scenario "Manifest gates authority" with denied-effect fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/ability-manifest-denial.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V4 Verify scenario "Replay handler denies unexpected effects" with trace mismatch fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/ability-replay-denial.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V5 Verify scenario "Name resolves to immutable hash" with pointer inspect fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/name-resolves-hash.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V6 Verify scenario "Pointer update supports rollback" with pointer rollback fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/name-pointer-rollback.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V7 Verify scenario "Session records are typed and versioned" with session DTO fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-record-versioning.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V8 Verify scenario "Migration preserves replay identity" with migration fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-migration-identity.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V9 Verify scenario "Session todo state is structured" with todo-state export fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-todo-structured.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V10 Verify scenario "Resolved and unresolved work remain distinguishable" with state transition fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-todo-statuses.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V11 Verify scenario "Pure cache hit reuses matching result" with cache hit fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/pure-cache-hit.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)
- [x] V12 Verify scenario "Environment drift invalidates cache" with cache miss fixture; save output at `openspec/changes/add-unison-inspired-agent-roadmap/evidence/pure-cache-env-drift.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-unison-inspired-agent-roadmap proposal/design/specs against Unison-inspired source concepts and Slotcar Rust/OpenSpec constraints`; input=`openspec/changes/add-unison-inspired-agent-roadmap/proposal.md, design.md, specs/consumer-integration/spec.md, specs/documentation/spec.md, specs/performance/spec.md`; asserts=`scope imports content identity, ability contracts, pointer names, typed sessions, structured todos, and pure caching without copying Unison syntax or runtime semantics`; evidence=`openspec/changes/add-unison-inspired-agent-roadmap/evidence/scope-review.txt` ✅ 9m 9s (started: 2026-05-17T04:13:45Z → completed: 2026-05-17T04:22:54Z)

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Hash normalized artifacts` -> `V1` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/artifact-hash-normalization.txt`
- `Scenario: Inspect and replay by hash` -> `V2` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/artifact-inspect-replay.txt`
- `Scenario: Manifest gates authority` -> `V3` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/ability-manifest-denial.txt`
- `Scenario: Replay handler denies unexpected effects` -> `V4` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/ability-replay-denial.txt`
- `Scenario: Name resolves to immutable hash` -> `V5` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/name-resolves-hash.txt`
- `Scenario: Pointer update supports rollback` -> `V6` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/name-pointer-rollback.txt`
- `Scenario: Session records are typed and versioned` -> `V7` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-record-versioning.txt`
- `Scenario: Migration preserves replay identity` -> `V8` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-migration-identity.txt`
- `Scenario: Session todo state is structured` -> `V9` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-todo-structured.txt`
- `Scenario: Resolved and unresolved work remain distinguishable` -> `V10` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/session-todo-statuses.txt`
- `Scenario: Pure cache hit reuses matching result` -> `V11` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/pure-cache-hit.txt`
- `Scenario: Environment drift invalidates cache` -> `V12` -> `openspec/changes/add-unison-inspired-agent-roadmap/evidence/pure-cache-env-drift.txt`
