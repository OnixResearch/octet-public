# Law-tested deterministic cores

Octet law tests name deterministic pure-core invariants and keep generated inputs bounded and replayable. The current suites use checked-in case matrices instead of a random runner, so a failing case is replayed by rerunning the focused test name from the Nix devshell.

## Replay commands

```bash
nix develop -c bash -c 'cargo test --lib config_law -- --nocapture'
nix develop -c bash -c 'cargo test -p valence-core evidence_index_law -- --nocapture'
```

## Law catalog

| Core | Law | Generated domain | Budget | Negative coverage |
| --- | --- | --- | ---: | --- |
| `src/config.rs` lint-level parser/validator | Valid per-lint levels normalize deterministically. | `no_unwrap` and `no_panic` crossed with `allow`, `warn`, and `deny`. | 6 cases | Invalid levels (`fatal`, `Deny`, ` warn `) return typed `InvalidLintLevel` diagnostics and no accepted override. |
| `src/config.rs` config precedence | `disabled_lints` wins over `lint_levels`. | `no_unwrap` and `no_panic` crossed with every valid level. | 6 cases | Conflicting disabled/level entries return stable `ConflictingDisabledLintLevel` diagnostics and keep the lint disabled. |
| `crates/valence-core` evidence codebase index | Equivalent inputs have byte-stable identity. | Reversed input order and two absolute-root locator variants. | 3 cases | Conflicting duplicate locators with malformed JSON fail closed with strict-mode errors and stable ordering. |
| `crates/valence-core` evidence codebase index | Records and diagnostics remain deterministically ordered. | Every generated evidence-index law case. | shared | Ordering assertions run on positive and negative generated cases. |

Concrete examples, UI fixtures, golden output, transcript evidence, and Nix rails remain authoritative for user-visible behavior. Law tests only assert pure-core invariants that should survive refactors.
