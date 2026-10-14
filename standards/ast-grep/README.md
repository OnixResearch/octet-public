# Octet ast-grep structural rail

This rule pack emits syntax-aware candidate evidence. It does not replace Dylint, rustc, Verus, Kani, proof rails, or release-readiness gates.

## Profiles and promotion

`profile.ncl` is the typed source of truth. `generated/profile.json` is the runtime export. The profile declares inventory, advisory, warning, error, and migration lanes. Every rule outside inventory requires positive and negative rule tests.

The bundle BLAKE3 binds `sgconfig.yml`, declared rule and test files, and the ordered scan paths/globs. `cargo octet evidence collect --rail ast-grep --run --config standards/ast-grep/generated/profile.json` recomputes that identity before running the pinned ast-grep cohort.

## Codemods

Codemods are dry-run-only by default. A migration must preserve its proposed diff and focused validation commands. No rule in this initial pack carries an automatic fixer.

## Validation

```console
ast-grep test --config standards/ast-grep/sgconfig.yml
ast-grep scan --rule standards/ast-grep/rules/unsafe-block-review.yml --json=compact standards/ast-grep/fixtures/positive
ast-grep scan --rule standards/ast-grep/rules/unsafe-block-review.yml --json=compact standards/ast-grep/fixtures/negative
cargo test -p cargo-octet ast_grep
```
