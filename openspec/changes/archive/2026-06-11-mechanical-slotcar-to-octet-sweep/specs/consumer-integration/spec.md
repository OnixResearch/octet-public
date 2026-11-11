## MODIFIED Requirements

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
