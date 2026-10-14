## ADDED Requirements

### Requirement: Content Addressed Artifact Store [r[content-addressed-artifact-store]]

The system MUST define a content-addressed artifact store for prompts, tool schemas, model requests, session blocks, receipts, and skill/tool definitions. Artifact identities MUST use deterministic normalized serialization and `b3:` BLAKE3 hashes, and MUST exclude raw secrets, absolute paths, timestamps, local usernames, and environment dumps.

#### Scenario: Hash normalized artifacts [r[content-addressed-artifact-store.scenario.hash-normalized-artifacts]]

- GIVEN two semantically identical artifact DTOs that differ only by whitespace, field order, or local path presentation
- WHEN the artifact store computes identity
- THEN both DTOs produce the same `b3:` identity
- AND the stored hash input excludes local-machine-only fields

#### Scenario: Inspect and replay by hash [r[content-addressed-artifact-store.scenario.inspect-replay-by-hash]]

- GIVEN a stored model request or tool schema artifact referenced by hash
- WHEN an operator runs the inspect or replay path for that hash
- THEN the output names the schema version, artifact kind, hash, and replay prerequisites
- AND replay fails closed if a referenced dependency hash is missing

### Requirement: Tool Ability Contracts [r[tool-ability-contracts]]

The system MUST define ability-style tool/effect contracts for agent tool calls. The initial effect vocabulary MUST include file read, file write, shell, network, secret, browser, scheduler, and remote execution authority. Enforcement and replay receipts MUST record the effective authority set without recording raw secrets.

#### Scenario: Manifest gates authority [r[tool-ability-contracts.scenario.manifest-gates-authority]]

- GIVEN a tool call declares only file-read authority
- WHEN the call attempts network, shell, secret, or file-write effects
- THEN the handler denies the undeclared effect
- AND the denial receipt names the missing authority without exposing secret values or local absolute paths

#### Scenario: Replay handler denies unexpected effects [r[tool-ability-contracts.scenario.replay-denies-unexpected-effects]]

- GIVEN a recorded replay trace for a tool call with a fixed effect manifest
- WHEN replay observes an effect not present in the trace or manifest
- THEN replay fails deterministically
- AND the failure identifies the unexpected effect family

### Requirement: Name Hash Pointer Registry [r[name-hash-pointer-registry]]

The system MUST support a registry where human-friendly names resolve to immutable artifact hashes. Pointer updates MUST create deterministic receipts that include name, old hash, new hash, target artifact kind, and rollback hint.

#### Scenario: Name resolves to immutable hash [r[name-hash-pointer-registry.scenario.name-resolves-to-immutable-hash]]

- GIVEN a name such as a skill, prompt, tool, or model profile alias points at an artifact hash
- WHEN the name is inspected
- THEN the output shows the current target hash and target artifact kind
- AND the underlying artifact content is addressed by hash rather than by the mutable name

#### Scenario: Pointer update supports rollback [r[name-hash-pointer-registry.scenario.pointer-update-supports-rollback]]

- GIVEN a name pointer is updated from one artifact hash to another
- WHEN the update receipt is emitted
- THEN the receipt includes old hash, new hash, name, target kind, and rollback command or operation hint
- AND rollback restores the old hash without mutating either artifact definition

### Requirement: Typed Durable Session Records [r[typed-durable-session-records]]

The system MUST persist session blocks, model requests, tool calls, tool outputs, findings, review decisions, and todo state as versioned typed records suitable for replay, migration, and semantic search. Durable records MUST carry schema version and redaction state, and MUST avoid raw secrets and unstable local environment data.

#### Scenario: Session records are typed and versioned [r[typed-durable-session-records.scenario.records-typed-versioned]]

- GIVEN a session contains model requests, tool calls, outputs, findings, and review decisions
- WHEN the session is persisted
- THEN each durable record names its kind, schema version, stable identity, and redaction status
- AND raw log text is stored only as an attachment or bounded handle when needed

#### Scenario: Migration preserves replay identity [r[typed-durable-session-records.scenario.migration-preserves-replay-identity]]

- GIVEN an older session record schema is migrated to a newer schema
- WHEN the migration runs
- THEN the migrated record preserves the original artifact identity or records an explicit identity-change receipt
- AND replay can distinguish schema migration from changed tool/model behavior
