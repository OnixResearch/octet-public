## ADDED Requirements

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
