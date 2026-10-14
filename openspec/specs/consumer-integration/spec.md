# Consumer Integration Specification

## Purpose

Defines the structured output contract for `cargo tigerstyle check`, enabling machine-readable diagnostics for LLM-driven triage, CI integration (SARIF), and human progress tracking (summary).
## Requirements
### Requirement: JSON Diagnostic Output
The system MUST support a `--output-format json` flag that emits one JSON object per diagnostic line to stdout, while rustc's human-readable output continues on stderr.

Each JSON object MUST contain the following fields with the specified types:
- `index`: string — finding identifier (`F1`, `F2`, ...) unique within the run
- `lint`: string — bare lint name (e.g., `no_unwrap`)
- `group`: string — domain group, one of `tigerstyle::safety`, `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`, `tigerstyle::concurrency`, or `tigerstyle::unknown` for unrecognized lints. This is an output-side label resolved from a static lookup table (`DIAGNOSTIC_METADATA` in `src/diagnostics.rs`), not dependent on Dylint/rustc group registration
- `severity`: string — effective severity, one of `warning`, `error`
- `file`: string — relative file path from workspace root
- `line`: integer — 1-based line number
- `column`: integer — 1-based column number
- `message`: string — primary diagnostic message text
- `suggestion`: string or null — suggested replacement text if the lint emits one
- `applicability`: string or null — one of `MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, or null
- `principle`: string — Tiger Style principle name sourced from `DIAGNOSTIC_METADATA` in `src/diagnostics.rs` (e.g., `Assertion Quality`, `Function Length`). If a lint fires that is not registered in `DIAGNOSTIC_METADATA`, the value MUST be `"unknown"`
- `crate_name`: string — name of the crate being checked, extracted from the `target.name` field in cargo's `compiler-message` JSON wrapper
- `version`: integer — schema version, initially `1`; incremented on breaking field changes

#### Scenario: LLM parses JSON findings into granular tasks
- GIVEN a workspace with findings from multiple lints across multiple crates
- WHEN `cargo tigerstyle check --output-format json` completes
- THEN stdout contains one valid JSON object per line, each parseable with `serde_json::from_str`, and every object has all required fields

#### Scenario: JSON output coexists with human stderr
- GIVEN `--output-format json` is specified
- WHEN the check runs
- THEN JSON findings appear on stdout (one per line), and stderr contains only the trailing summary block (no human-readable diagnostics in `json` mode, since stdout carries the structured data)

### Requirement: SARIF Output
The system MUST support a `--output-format sarif` flag that emits a SARIF v2.1.0 document to stdout.

The SARIF document MUST include:
- `$schema` set to the SARIF v2.1.0 JSON schema URI
- `version` set to `"2.1.0"`
- `runs` array with exactly one run object containing:
  - `tool.driver.name` set to `"cargo-tigerstyle"`
  - `tool.driver.version` set to the cargo-tigerstyle version
  - `tool.driver.rules[]` with one entry per distinct lint that fired, each with `id` (bare lint name), `shortDescription`, and a `properties.group` field
  - `results[]` with one entry per finding, each containing:
    - `ruleId` set to the bare lint name
    - `level` set to `"warning"` or `"error"`
    - `message.text` matching the primary diagnostic message
    - `locations[0].physicalLocation` with `artifactLocation.uri` (relative file path), `region.startLine`, and `region.startColumn`
    - `properties.findingIndex` set to the finding index string (e.g., `"F3"`)

#### Scenario: GitHub Code Scanning ingestion
- GIVEN a workspace with findings
- WHEN `cargo tigerstyle check --output-format sarif` completes
- THEN the stdout output is valid JSON conforming to SARIF v2.1.0 structure with `$schema`, `version`, `runs[0].tool.driver.name`, `runs[0].tool.driver.rules[]`, and `runs[0].results[]` fields present. Validation is structural spot-checks in Rust integration tests, not full JSON Schema validation

### Requirement: Run Summary
The system MUST print a closing summary to stderr at the end of every check run, regardless of output format.

The summary MUST include:
- `--- tigerstyle summary ---` opening delimiter and `---` closing delimiter for machine extraction
- total finding count
- per-crate finding count (sorted descending by count)
- per-lint finding count (sorted descending by count)
- count of autofixable findings (those with `MachineApplicable` suggestions)
- per-finding index table listing each finding's index, lint, crate, and file:line

#### Scenario: Summary after noisy run
- GIVEN a workspace with 15 findings across 3 crates from 4 different lints
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary block showing crate counts, lint counts, total, and autofixable count

#### Scenario: Summary for clean run
- GIVEN a workspace with zero findings
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary showing `Findings: 0` (no autofixable parenthetical for zero findings)

### Requirement: Error Path Behavior
The system MUST produce valid output on stdout even when the underlying `cargo check` encounters compilation errors (exit code 2 path).

- In `json` mode: any findings emitted before the error MUST appear as valid JSON lines; no partial/malformed JSON on stdout
- In `sarif` mode: the SARIF document MUST still be a valid JSON document with whatever findings were emitted before the error
- In all modes: the summary MUST still print on stderr, reflecting only the findings that were successfully emitted
- The exit code MUST be `2` when the underlying cargo check fails, regardless of output format

#### Scenario: Compilation error during JSON output
- GIVEN a workspace with a syntax error in one crate and lint findings in another
- WHEN `cargo tigerstyle check --output-format json` runs
- THEN stdout contains valid JSON lines for findings emitted before the error, stderr contains the compilation error, and exit code is 2

#### Scenario: Compilation error during SARIF output
- GIVEN a workspace with a compilation error
- WHEN `cargo tigerstyle check --output-format sarif` runs
- THEN stdout is a valid SARIF JSON document (possibly with zero results), and exit code is 2

### Requirement: Finding Index
The system MUST assign each finding a deterministic index (`F1`, `F2`, ...) based on order of appearance in the diagnostic stream.

The index MUST be:
- unique within a single run
- monotonically increasing
- assigned by order of arrival from the cargo subprocess diagnostic stream (not sorted)
- present in all output formats (JSON field, SARIF property, and prefixed in human summary)

Indices are run-scoped only and NOT stable across invocations. In multi-crate workspaces, cargo may emit diagnostics in different orders due to parallel compilation, so the same finding may receive different indices between runs.

#### Scenario: LLM references specific finding by index
- GIVEN a run that emits 10 findings
- WHEN an LLM reads the JSON output
- THEN it can reference `F3` and the index maps to exactly one finding with a specific file, line, and lint

### Requirement: Output Format Flag
The system MUST accept `--output-format {human,json,sarif}` on the `check` subcommand.

The default MUST be `human`, which preserves the current pass-through behavior identically.

#### Scenario: Default behavior unchanged
- GIVEN no `--output-format` flag is specified
- WHEN `cargo tigerstyle check` runs
- THEN the lint diagnostic messages on stderr are identical to the current behavior (rustc human-readable diagnostics with colors preserved on TTY), exit code is unchanged, and a new trailing summary block is appended to stderr after the diagnostic stream. Note: cargo progress/build-status output may differ slightly since the engine uses `--message-format=json` internally

#### Scenario: Invalid format rejected
- GIVEN `--output-format xml` is specified
- WHEN `cargo tigerstyle check` runs
- THEN the tool exits with code 2 and an error message listing valid formats

#### Scenario: Conflicting --message-format rejected
- GIVEN `cargo tigerstyle check -- --message-format=short` is specified
- WHEN the tool runs
- THEN the tool exits with code 2 and an error message explaining that `--message-format` is managed by `--output-format`

### Requirement: Fingerprint Command Surface [r[fingerprint-command-surface]]

The system MUST provide a `cargo tigerstyle fingerprint` command surface for validating and maintaining selected function fingerprints.

The command MUST support `--check`, `--write`, `--update <path-or-filter>`, and `--diff` modes. Check mode MUST fail on mismatches and invalid metadata. Write and update modes MUST modify only fingerprint attributes or marker comments, not function bodies. Diff mode MUST list reviewed functions whose computed hashes differ from stored hashes.

#### Scenario: Check reports matching fingerprints [r[fingerprint-command-surface.scenario.check-matching]]

- GIVEN a workspace with fingerprinted functions whose stored hashes match computed hashes
- WHEN `cargo tigerstyle fingerprint --check` runs
- THEN the command exits successfully
- AND it reports the number of checked functions

#### Scenario: Check fails on mismatch [r[fingerprint-command-surface.scenario.check-mismatch]]

- GIVEN a fingerprinted function whose body tokens changed after review
- WHEN `cargo tigerstyle fingerprint --check` runs
- THEN the command exits non-zero
- AND the diagnostic includes the function name, expected hash, actual hash, and update command hint

#### Scenario: Write mode updates markers only [r[fingerprint-command-surface.scenario.write-markers-only]]

- GIVEN a Rust file with a missing or stale function fingerprint marker
- WHEN `cargo tigerstyle fingerprint --write` runs
- THEN the command writes or updates the fingerprint marker
- AND it does not change the function body tokens

#### Scenario: Diff mode lists reviewed changes [r[fingerprint-command-surface.scenario.diff-reviewed-functions]]

- GIVEN multiple fingerprinted functions and one changed reviewed function
- WHEN `cargo tigerstyle fingerprint --diff` runs
- THEN the output lists the changed function, file path, old hash, new hash, and review class
- AND unchanged functions are not listed as changed

### Requirement: Fingerprint Marker Forms [r[fingerprint-marker-forms]]

The system MUST support both first-class attribute markers and stable comment markers for function fingerprints.

Attribute form MUST support `#[fingerprint("b3:...", review = "...")]` or an equivalent Tigerstyle-owned proc-macro path. Comment form MUST support an adjacent marker such as `// tigerstyle-fingerprint: b3:... review=...` immediately before the function. The checker MUST ignore its own marker when recomputing the function hash.

#### Scenario: Attribute marker validates [r[fingerprint-marker-forms.scenario.attribute-validates]]

- GIVEN a function annotated with a supported fingerprint attribute
- WHEN fingerprint validation runs
- THEN the checker parses the hash and review metadata from the attribute
- AND it validates the computed function hash against the stored hash

#### Scenario: Comment marker validates without proc macro [r[fingerprint-marker-forms.scenario.comment-validates]]

- GIVEN a stable Rust crate that uses an adjacent `// tigerstyle-fingerprint:` marker instead of a proc macro attribute
- WHEN fingerprint validation runs
- THEN the checker parses and validates the marker
- AND the crate does not need to add a fingerprint proc-macro dependency solely for validation

### Requirement: Fingerprint Machine Output [r[fingerprint-machine-output]]

The system MUST provide deterministic machine-readable output for fingerprint validation results when requested.

Machine output MUST include the schema version, function name, relative file path, review class, stored hash, computed hash, status, macro allowance state, and optional evidence or spec reference. It MUST NOT include absolute paths, timestamps, local usernames, or environment dumps.

#### Scenario: Machine output captures mismatch identity [r[fingerprint-machine-output.scenario.mismatch-identity]]

- GIVEN `cargo tigerstyle fingerprint --check --output-format json` finds a mismatch
- WHEN the command emits machine output
- THEN the JSON record includes function name, relative path, review class, stored hash, computed hash, and status `mismatch`
- AND it omits absolute paths and timestamps

#### Scenario: Inventory preserves evidence references [r[fingerprint-machine-output.scenario.inventory-evidence]]

- GIVEN fingerprint metadata includes a spec ID or evidence path
- WHEN machine inventory output is requested
- THEN the JSON record includes that reference unchanged
- AND the record remains deterministic across repeated runs with the same source tree

### Requirement: Content Addressed Finding Identity [r[content-addressed-finding-identity]]

The system MUST support optional content-addressed identity fields for Tigerstyle findings, including finding fingerprint, source hash, snippet hash, config hash, and tool revision.

#### Scenario: Finding hash fields appear on request [r[content-addressed-finding-identity-scenario-finding-hash-fields-appear-on-request]]

- GIVEN JSON output is requested with content-addressed fields enabled
- WHEN a lint finding is emitted
- THEN the JSON record includes stable hash fields without absolute paths or timestamps

#### Scenario: Line moves do not define identity [r[content-addressed-finding-identity-scenario-line-moves-do-not-define-identity]]

- GIVEN a finding moves line number while its normalized snippet and lint context remain stable
- WHEN fingerprints are compared
- THEN the finding identity remains attributable without relying on the old line number

### Requirement: Content Addressed Evidence Headers [r[content-addressed-evidence-headers]]

The system MUST define evidence transcript headers that pin command, config, tool revision, fixture hashes, and repository revision without recording secrets or unstable local environment values.

#### Scenario: Evidence header is deterministic [r[content-addressed-evidence-headers-scenario-evidence-header-is-deterministic]]

- GIVEN the same command, config, fixtures, tool revision, and repo revision
- WHEN an evidence header is regenerated
- THEN the header hashes are identical across repeated runs

### Requirement: Semantic Refactor Evidence Records [r[semantic-refactor-evidence-records]]

The system MUST define evidence records for rename and move changes that preserve symbol identity with old path, new path, symbol name, optional fingerprint, and review rationale.

#### Scenario: Move record preserves identity [r[semantic-refactor-evidence-records-scenario-move-record-preserves-identity]]

- GIVEN a reviewed function moves files without token changes
- WHEN semantic refactor evidence is generated
- THEN the record links old path, new path, symbol name, and fingerprint so reviewers can see identity preservation

#### Scenario: Rename record states rationale [r[semantic-refactor-evidence-records-scenario-rename-record-states-rationale]]

- GIVEN a symbol is renamed
- WHEN refactor evidence is captured
- THEN the record includes old name, new name, path context, and rationale rather than relying on diff noise alone

### Requirement: Refactor Evidence Machine Output [r[refactor-evidence-machine-output]]

The system MUST provide deterministic machine-readable refactor evidence that omits absolute paths, timestamps, usernames, and environment dumps.

#### Scenario: Machine output is stable [r[refactor-evidence-machine-output-scenario-machine-output-is-stable]]

- GIVEN the same refactor mapping and source tree
- WHEN refactor evidence is emitted twice
- THEN the output is identical except for intentionally changed records

### Requirement: Octet Canonical CLI Surface

The system MUST expose `cargo octet` / `cargo-octet` as the canonical downstream command surface for check runs, structured output, summaries, SARIF tool metadata, Nix package/app outputs, and consumer templates. If legacy `cargo slotcar` / `cargo-slotcar` or `cargo tigerstyle` / `cargo-tigerstyle` paths remain available during transition, they MUST be documented as compatibility aliases and MUST delegate to the same implementation and output schema as `cargo octet`.

The deterministic proof artifact `openspec/changes/archive/2026-05-18-rename-slotcar-to-octet/evidence/cli-surface.txt` MUST show the canonical binary/package surfaces include Octet and that any supported compatibility alias exits successfully with equivalent behavior for a minimal fixture.

#### Scenario: Cargo subcommand uses octet

- GIVEN a consumer workspace with the repo-provided tool installed from this checkout
- WHEN the consumer runs `cargo octet check`
- THEN the command executes the same lint pipeline as the prior first-class cargo wrapper and reports the tool identity as `octet`

#### Scenario: Structured outputs name octet

- GIVEN a check run emits human summary, JSON, or SARIF output
- WHEN a consumer inspects tool-name fields, delimiters, or headings in the output
- THEN product/tool identity fields use `octet` instead of legacy names, except for explicitly documented compatibility or lint-namespace fields

#### Scenario: Compatibility alias is explicit

- GIVEN the implementation keeps `cargo slotcar` or `cargo tigerstyle` available during the rename transition
- WHEN a consumer invokes that legacy command path
- THEN it is documented and tested as a compatibility alias to the canonical `cargo octet` behavior rather than a second independent tool surface

### Requirement: Effect Surface Reporting [r[effect-surface-reporting]]

The system MUST classify Octet findings into reviewable effect families such as `time`, `failure`, `resource`, `concurrency`, `io`, `remote`, `randomness`, `unsafe`, or `unknown`. Machine outputs MUST include the effect family, and human summaries MUST aggregate findings by effect family.

#### Scenario: JSON findings include effect family [r[effect-surface-reporting.scenario.json-findings-include-effect-family]]

- GIVEN JSON output is requested for findings from safety, purity, and concurrency lints
- WHEN `cargo octet check --output-format json` emits findings
- THEN each finding includes an `effect_family` field resolved from static diagnostic metadata

#### Scenario: Human summary aggregates effects [r[effect-surface-reporting.scenario.human-summary-aggregates-effects]]

- GIVEN a run emits findings across multiple effect families
- WHEN the check summary is printed
- THEN the summary includes deterministic per-effect counts sorted by count then family name

#### Scenario: Unknown lints stay explicit [r[effect-surface-reporting.scenario.unknown-lints-stay-explicit]]

- GIVEN a diagnostic is emitted for a lint missing effect metadata
- WHEN machine output is generated
- THEN the finding uses `effect_family: "unknown"` and the metadata coverage check fails in repo tests

### Requirement: Semantic Review Identity [r[semantic-review-identity]]

The system MUST provide a semantic review identity for fingerprinted Rust items and lint findings that treats names, file paths, and line numbers as metadata rather than the sole identity source. The identity MUST include deterministic hashes for normalized item shape, lint/config context, Octet tool version, and dependency identity where available. It MUST NOT include absolute paths, usernames, timestamps, or secrets.

#### Scenario: Normalized item identity survives rename [r[semantic-review-identity.scenario.normalized-item-identity-survives-rename]]

- GIVEN a fingerprinted function is renamed without changing its normalized body shape or dependency identity
- WHEN `cargo octet fingerprint --check --output-format json` emits the record
- THEN the semantic identity stays stable while name metadata changes

#### Scenario: Baseline matching uses semantic identity first [r[semantic-review-identity.scenario.baseline-matching-uses-semantic-identity-first]]

- GIVEN a lint finding moves to a new line after a behavior-preserving refactor
- WHEN baseline comparison runs with semantic identity fields present
- THEN the finding is attributed by semantic identity before falling back to path and line

#### Scenario: Dependency identity is explicit when unavailable [r[semantic-review-identity.scenario.dependency-identity-is-explicit-when-unavailable]]

- GIVEN a function calls code whose dependency identity cannot be computed by the source scanner
- WHEN the review manifest is generated
- THEN the manifest records an explicit `dependency_identity_status` instead of silently pretending dependencies were hashed

### Requirement: Structured Rollout Migration Sessions [r[structured-rollout-migration-sessions]]

The system MUST support structured rollout migration sessions that record lint rollout state as deterministic manifests rather than requiring consumers to triage an unstructured diagnostic stream. A session MUST capture profile, command inputs, config hash, tool revision, finding identities, accepted findings, autofix actions, deferred findings, and manual-review checkpoints.

#### Scenario: Migration start writes deterministic manifest [r[structured-rollout-migration-sessions.scenario.migration-start-writes-deterministic-manifest]]

- GIVEN a consumer workspace requests a starter-profile migration
- WHEN `cargo octet migrate start --profile starter` runs
- THEN a migration manifest is written with profile, tool revision, config hash, and initial finding identities

#### Scenario: Migration status reports remaining work [r[structured-rollout-migration-sessions.scenario.migration-status-reports-remaining-work]]

- GIVEN a migration manifest contains accepted, fixed, deferred, and unresolved findings
- WHEN `cargo octet migrate status --format json` runs
- THEN the output reports deterministic counts and unresolved work without reclassifying accepted findings

#### Scenario: Safe apply does not modify manual-review findings [r[structured-rollout-migration-sessions.scenario.safe-apply-does-not-modify-manual-review-findings]]

- GIVEN a migration has both machine-applicable findings and findings marked manual-review
- WHEN `cargo octet migrate apply-safe` runs
- THEN only machine-applicable changes are applied and manual-review findings remain pending

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

### Requirement: Consumer Effect Capability Summary [r[consumer-effect-capability-summary]]

The system MUST expose Octet effect capability summaries in consumer-facing outputs so downstream teams can triage ambient authority by capability family.

#### Scenario: Consumer output groups findings by capability [r[consumer-effect-capability-summary.scenario.groups-findings]]

- GIVEN a downstream consumer runs Octet with JSON or human output
- WHEN findings include mapped effect capability families
- THEN the summary groups counts by capability family
- AND each grouped family can be traced back to the contributing lint names

#### Scenario: Consumer output preserves advisory status [r[consumer-effect-capability-summary.scenario.preserves-advisory-status]]

- GIVEN a capability family is advisory rather than a hard gate
- WHEN the consumer output is generated
- THEN the output preserves that advisory status
- AND it does not turn advisory capability observations into failing exit codes without a separate hard-rule policy

### Requirement: Octet Replay Capsule Capture [r[octet-replay-capsule-capture]]

The system MUST support a replay-capsule capture contract for Octet lint and fingerprint runs. A capsule MUST record capsule schema version, repo revision or dirty-state summary, Octet binary identity, command argv, config hash, selected input file hashes, output artifact hashes, and redacted environment allowlist.

#### Scenario: Capture pins check run identity [r[octet-replay-capsule-capture.scenario.check-run-identity]]

- GIVEN `cargo octet check` runs with JSON or SARIF output and an artifact directory
- WHEN replay-capsule capture is requested
- THEN the capsule records the command argv, Octet binary identity, config hash, selected source hashes, and output artifact hashes
- AND the capsule does not include raw secrets, absolute home paths, local usernames, or unbounded environment dumps

#### Scenario: Capture pins fingerprint run identity [r[octet-replay-capsule-capture.scenario.fingerprint-run-identity]]

- GIVEN `cargo octet fingerprint --check --review-manifest <path>` runs against selected source files
- WHEN replay-capsule capture is requested
- THEN the capsule records the review manifest hash, selected source hashes, fingerprint output hash, and fingerprint command argv
- AND the capsule keeps review metadata traceable without claiming the fingerprint proves semantic correctness

### Requirement: Octet Replay Capsule Replay [r[octet-replay-capsule-replay]]

The system MUST replay a captured Octet capsule against the current workspace or an explicit checkout and report whether observed inputs and outputs match the capsule. Replay MUST distinguish exact matches, expected-failure matches, and drift.

#### Scenario: Replay accepts exact match [r[octet-replay-capsule-replay.scenario.exact-match]]

- GIVEN a replay capsule captured from a clean Octet run
- WHEN the same command, config, selected inputs, and output artifacts are replayed unchanged
- THEN replay reports `match`
- AND the report names the capsule hash and every matched artifact hash

#### Scenario: Replay reports drift [r[octet-replay-capsule-replay.scenario.drift]]

- GIVEN a replay capsule with pinned source and output hashes
- WHEN a selected input, config, Octet binary identity, or output artifact differs
- THEN replay reports `drift`
- AND the report names the mismatched field and expected/observed hash values

#### Scenario: Replay accepts expected failure [r[octet-replay-capsule-replay.scenario.expected-failure]]

- GIVEN a replay capsule declares an expected nonzero exit code and expected output hash
- WHEN replay observes the same exit code and output hash
- THEN replay reports `expected_failure_match`
- AND the report does not treat that expected failure as a clean successful lint run

### Requirement: Octet Consumer Command Surface

The system MUST provide `cargo octet` / `cargo-octet` as the canonical consumer command surface for check, fingerprint, migrate, roadmap, and transcript workflows. User-facing help, structured outputs, SARIF tool metadata, artifact sidecars, summaries, migration manifests, transcript reports, fingerprints, package outputs, config examples, and Nix apps MUST identify the canonical tool as Octet / `cargo-octet`.

For this mechanical sweep, active command, package, config, environment, receipt, and fixture surfaces MUST contain no unapproved `slotcar` product identity after the Steel apply pass. Retained `slotcar` aliases, fallback parsers, fixture fields, or Nix outputs MUST be listed in the Steel report as `allowed-compatibility` with a rationale and focused test coverage; otherwise they MUST be retired or renamed to Octet in the same change.

#### Scenario: Steel sweep handles command and package surfaces

- GIVEN active Cargo, Nix, runner, standards, and fixture files contain `slotcar` command or package names
- WHEN the Steel sweep applies approved replacements
- THEN canonical command, binary, package, Nix app, and structured-output identity strings use `octet` / `cargo-octet`
- AND any path or crate rename required by those references is updated consistently

#### Scenario: Compatibility retirement or allowlist is explicit

- GIVEN a `cargo slotcar`, `[slotcar]`, `workspace.metadata.slotcar`, `slotcar::`, `slotcar-allow`, or `slotcar_binary_identity` surface remains after the sweep
- WHEN deterministic verification inspects the Steel report and focused tests
- THEN the occurrence is either covered by an `allowed-compatibility` rationale plus a passing test or has been retired from code, docs, templates, fixtures, and Nix outputs

#### Scenario: Octet config and command paths stay functional

- GIVEN the mechanical sweep updated active command and config surfaces
- WHEN focused CLI, config, and standards checks run
- THEN `cargo octet` help/config behavior and Octet standards outputs still pass without requiring a `slotcar` spelling for new consumers

### Requirement: Octet Consumer Configuration Surface

The system MUST support consumer-owned Octet configuration using `[octet]`, `[octet.lint_levels]`, and `[workspace.metadata.octet]` as canonical sections. During migration, legacy `[slotcar]`, `[tigerstyle]`, `[workspace.metadata.slotcar]`, or equivalent legacy sections MAY be accepted only through an explicit precedence rule that prefers Octet when both current and legacy sections are present.

Templates and new documentation MUST show Octet configuration first. Any retained legacy examples MUST be labeled as migration compatibility.

#### Scenario: Octet config section controls lint behavior

- GIVEN a consumer workspace with `[octet]` in `dylint.toml`
- WHEN `cargo octet check` runs
- THEN threshold, disabled-lint, allowlist, and lint-level settings are read from the Octet section

#### Scenario: Octet workspace metadata controls runner policy

- GIVEN a consumer workspace `Cargo.toml` with `[workspace.metadata.octet]`
- WHEN `cargo octet check` is invoked without explicit scope
- THEN the runner uses Octet metadata for default scope and cargo-check tail arguments

#### Scenario: Octet config wins over legacy config

- GIVEN both Octet and legacy configuration sections are present with different values
- WHEN the runner or lint library resolves configuration
- THEN the Octet section takes precedence
- AND the run emits or records a migration-compatible notice for the ignored legacy section when practical

### Requirement: Indexable Policy Self Dogfood Inventory [r[indexable-policy-self-dogfood-inventory]]

The system MUST provide saved dogfood evidence for running the archived
Indexable-derived lint family against the Octet repository itself.

The dogfood inventory MUST use the consumer-facing `cargo-octet` check path with JSON findings so maintainers can classify
findings by lint, file, and crate without parsing human diagnostics.

The inventory MUST enable the nine lints added by
`openspec/changes/archive/2026-05-18-add-indexable-policy-lints/`:
`drop_must_use`, `fallible_int_fallback`, `magic_number`,
`excessive_file_length`, `module_file_count`, `underscore_in_module_filename`,
`non_trait_imports`, `renamed_imports`, and `path_segment_repetition`.

#### Scenario: Dogfood run emits machine-readable finding inventory [r[indexable-policy-self-dogfood-inventory.scenario.machine-inventory]]

- GIVEN the Octet repository has the archived indexable-policy lints available
- WHEN the dogfood helper runs the consumer-facing check path with JSON output
- THEN it saves raw JSONL findings and a deterministic summary grouped by lint
  and file
- AND the summary names the exact command and exit code

#### Scenario: Dogfood triage records fixed or accepted findings [r[indexable-policy-self-dogfood-inventory.scenario.triage-recorded]]

- GIVEN the dogfood inventory reports one or more findings
- WHEN maintainers finish the slice
- THEN each finding is either fixed by a narrow source/config change or recorded
  as accepted debt with the finding index, lint name, file path, and rationale
- AND the triage evidence states whether the final rerun is clean or still has
  accepted findings
