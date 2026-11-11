## ADDED Requirements

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
