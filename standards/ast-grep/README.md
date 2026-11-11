# Octet ast-grep structural rail

This rule pack emits syntax-aware candidate evidence. It does not replace Dylint, rustc, Verus, Kani, proof rails, or release-readiness gates.

## Profiles and promotion

`profile.ncl` is the typed source of truth. `generated/profile.json` is the runtime export. The profile declares inventory, advisory, warning, error, and migration lanes. Every rule outside inventory requires positive and negative rule tests. Because `--config` loads the raw JSON export, `cargo octet` re-checks the `contracts.ncl` constraints it relies on: lane names, `severity_posture` (inventory, advisory, warning, or error), and non-empty scan globs.

The bundle BLAKE3 binds `sgconfig.yml`, declared rule and test files, and the ordered scan paths/globs. Each list (bundle inputs, scan paths, globs) is hashed with its entry count, so moving an entry between scan paths and globs changes the identity. `cargo octet evidence collect --rail ast-grep --run --config standards/ast-grep/generated/profile.json` recomputes that identity before running the pinned ast-grep cohort.

Importing a stored scan envelope (`cargo octet evidence collect --rail ast-grep --input scan.json --config standards/ast-grep/generated/profile.json`) also requires `--config`. The import recomputes the bundle identity from the checked-in profile and rule files, and requires the envelope's embedded profile to equal the checked-in profile field for field: tool version, bundle inputs and identity, scan scope, severity posture, promotion lanes, rule coverage, codemod policy, and non-claims. It refuses an envelope captured before a rule file, test, config, or any profile field changed, so those findings stay stale until a fresh scan receipt is captured.

## Codemods

Codemods are dry-run-only by default. A migration must preserve its proposed diff and focused validation commands. No rule in this initial pack carries an automatic fixer.

## Validation

```console
ast-grep test --config standards/ast-grep/sgconfig.yml
ast-grep scan --rule standards/ast-grep/rules/unsafe-block-review.yml --json=compact standards/ast-grep/fixtures/positive
ast-grep scan --rule standards/ast-grep/rules/unsafe-block-review.yml --json=compact standards/ast-grep/fixtures/negative
cargo test -p cargo-octet ast_grep
```
