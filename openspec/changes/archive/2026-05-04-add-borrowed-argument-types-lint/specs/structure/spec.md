## ADDED Requirements

### Requirement: Borrowed Argument Types Lint [r[borrowed-argument-types-lint]]

The lint MUST be able to warn on function and method parameters that take references to owned standard container types when a more general borrowed type communicates the same ownership contract.

The first implementation MUST cover `&String` with a suggestion to use `&str`, `&Vec<T>` with a suggestion to use `&[T]`, and `&PathBuf` with a suggestion to use `&Path`.

The lint MUST remain advisory or allow-by-default until compliant, noncompliant, and false-positive-boundary fixtures prove the intended behavior.

#### Scenario: String reference parameter [r[borrowed-argument-types-lint.scenario.string-reference]]

- GIVEN `fn parse_name(name: &String) -> Result<Name, Error>`
- WHEN the lint runs
- THEN it warns that `&String` exposes an owned container where `&str` is the clearer borrowed argument type
- AND the diagnostic suggests `&str` with review-required applicability

#### Scenario: Vec reference parameter [r[borrowed-argument-types-lint.scenario.vec-reference]]

- GIVEN `fn checksum(bytes: &Vec<u8>) -> u32`
- WHEN the lint runs
- THEN it warns that `&Vec<u8>` should be accepted as `&[u8]` when ownership and vector capacity are not part of the contract

#### Scenario: PathBuf reference parameter [r[borrowed-argument-types-lint.scenario.pathbuf-reference]]

- GIVEN `fn load_config(path: &PathBuf) -> Result<Config, Error>`
- WHEN the lint runs
- THEN it warns that `&PathBuf` should be accepted as `&Path` at borrowed path boundaries

#### Scenario: Trait dictated signature is skipped [r[borrowed-argument-types-lint.scenario.trait-dictated-skip]]

- GIVEN a trait method or trait implementation requires a parameter such as `fn visit(&self, value: &String)`
- WHEN the lint runs
- THEN it does NOT warn because the implementer may not control the signature

#### Scenario: Extern ABI signature is skipped [r[borrowed-argument-types-lint.scenario.extern-skip]]

- GIVEN an `extern "C"` function or callback boundary with a fixed ABI signature
- WHEN the lint runs
- THEN it does NOT warn because the signature belongs to an external boundary contract

#### Scenario: Already borrowed forms are accepted [r[borrowed-argument-types-lint.scenario.borrowed-forms-accepted]]

- GIVEN parameters typed as `&str`, `&[u8]`, and `&Path`
- WHEN the lint runs
- THEN it does NOT warn
