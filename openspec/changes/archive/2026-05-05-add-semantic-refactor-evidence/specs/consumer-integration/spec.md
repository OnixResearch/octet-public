## ADDED Requirements

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
