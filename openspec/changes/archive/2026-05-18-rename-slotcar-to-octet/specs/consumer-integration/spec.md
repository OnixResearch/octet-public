## ADDED Requirements

### Requirement: Octet Consumer Command Surface

The system MUST provide `cargo octet` / `cargo-octet` as the canonical consumer command surface for check, fingerprint, migrate, roadmap, and transcript workflows. User-facing help, structured outputs, SARIF tool metadata, artifact sidecars, summaries, migration manifests, transcript reports, and fingerprints MUST identify the canonical tool as Octet / `cargo-octet` unless they are explicitly documenting a transition alias.

The existing `cargo slotcar` and `cargo tigerstyle` command paths MAY remain during the migration, but only as deliberate compatibility aliases. Compatibility aliases MUST route to the same implementation as `cargo octet`, MUST be covered by focused tests or deterministic evidence, and MUST be documented as transition-only rather than canonical.

#### Scenario: Cargo subcommand uses Octet

- GIVEN a consumer has installed the renamed runner
- WHEN they run `cargo octet check --help`
- THEN help text names the canonical command as `cargo octet check`
- AND it does not instruct new users to prefer `cargo slotcar` or `cargo tigerstyle`

#### Scenario: Structured outputs name Octet

- GIVEN a consumer runs the check command with human, JSON, or SARIF output plus an artifact directory
- WHEN the output and sidecar files are inspected
- THEN tool names, command records, status fields, and SARIF driver metadata use Octet / `cargo-octet` for the canonical tool identity

#### Scenario: Compatibility aliases are explicit

- GIVEN a consumer invokes a retained `cargo slotcar` or `cargo tigerstyle` alias
- WHEN the command succeeds
- THEN the alias is documented or reported as compatibility behavior
- AND the resulting lint semantics and structured output schema match the canonical `cargo octet` path

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
