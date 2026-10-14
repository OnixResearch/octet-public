# Structured Migration Sessions Specification

## Purpose

Defines the `structured-migration-sessions` capability: reviewable Octet migration sessions that separate mutable names and locators from stable content identities while preserving compatibility and evidence boundaries.

## Requirements

### Requirement: Structured migration receipt

r[octet.structured_migration_sessions.receipt] Octet SHOULD emit structured migration session receipts with session ID, migration kind, old pointer, new pointer, identity evidence, compatibility classification, affected surfaces, planned operations, verification commands, observed status, and explicit non-claims.

#### Scenario: Valid receipt records review identity

r[octet.structured_migration_sessions.receipt.valid]

- GIVEN a migration session renames a lint, policy, config key, command alias, namespace, artifact locator, or evidence schema
- WHEN the session receipt is emitted
- THEN the receipt SHOULD record old and new pointers separately from BLAKE3-backed content or artifact identities where those identities exist.
- AND it SHOULD record affected docs, configs, source attrs, receipts, transcripts, generated artifacts, compatibility aliases, verification commands, and non-claim text.

#### Scenario: Invalid receipt fails closed

r[octet.structured_migration_sessions.receipt.invalid]

- GIVEN a migration receipt omits required pointer fields, names an unknown compatibility classification, omits required identity evidence for a strict migration, or names an unsupported migration kind
- WHEN receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field.

### Requirement: Migration planning and status

r[octet.structured_migration_sessions.plan] Octet SHOULD provide non-mutating plan and status output for structured migration sessions.

#### Scenario: Plan inventories affected surfaces

r[octet.structured_migration_sessions.plan.inventory]

- GIVEN a session declares a namespace, lint/policy, config, command, artifact-layout, or evidence-schema migration
- WHEN plan output is generated
- THEN it SHOULD list affected live surfaces, retained compatibility aliases, archive/read-only references, generated artifacts, required verification commands, and unresolved todo items.
- AND plan output MUST NOT mutate source files, docs, archives, or artifacts.

### Requirement: Migration verification

r[octet.structured_migration_sessions.verify] Octet SHOULD verify structured migration sessions by checking identity linkage, affected-surface coverage, compatibility alias coverage, artifact freshness, and removal-boundary completeness.

#### Scenario: Alias coverage is verified

r[octet.structured_migration_sessions.verify.alias_coverage]

- GIVEN a session changes a canonical namespace or command surface while retaining compatibility aliases
- WHEN migration verification runs
- THEN verification SHOULD pass only when every required live alias is covered by tests, docs, or explicit compatibility records.
- AND it SHOULD fail when a live alias lacks a removal boundary or blocking rationale.

#### Scenario: Archive boundary is preserved

r[octet.structured_migration_sessions.verify.archive_boundary]

- GIVEN a session inventory includes archived evidence, transcripts, or historical paths
- WHEN migration verification runs
- THEN archive/read-only surfaces SHOULD be reported separately from mutable live surfaces.
- AND verification MUST fail if a planned operation rewrites an archive surface without an explicit mutable classification and reviewer rationale.

### Requirement: Guarded migration application

r[octet.structured_migration_sessions.apply] Octet MAY apply selected safe migration operations only after a dry-run plan and pre-write operation manifest are available.

#### Scenario: Apply matches dry-run plan

r[octet.structured_migration_sessions.apply.dry_run_parity]

- GIVEN a selected migration operation is classified as safe for apply
- WHEN the operator runs dry-run and apply for the same session input
- THEN apply MUST write only operations listed in the dry-run plan and pre-write manifest.
- AND a repeated apply SHOULD be idempotent or fail closed with a deterministic already-applied status.

### Requirement: Migration claim boundaries

r[octet.structured_migration_sessions.boundaries] Structured migration sessions MUST state that they prove reviewed pointer and artifact linkage, not semantic equivalence or correctness.

#### Scenario: Migration avoids semantic-equivalence overclaim

r[octet.structured_migration_sessions.boundaries.no_semantic_equivalence]

- GIVEN migration docs, receipts, or status output describe a completed rename or artifact transition
- WHEN claim-boundary validation scans the output
- THEN the output MUST NOT claim that name-pointer migration proves behavioral correctness, semantic equivalence, source compatibility, or release safety without separate evidence.
