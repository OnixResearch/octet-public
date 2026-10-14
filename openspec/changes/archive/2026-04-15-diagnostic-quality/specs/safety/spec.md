## MODIFIED Requirements

### Requirement: Panic Macro In Production Code

The system MUST detect `panic!()`, `todo!()`, `unimplemented!()`, and
`unreachable!()` in production code.

The system MUST continue to support a narrow temporary-stub escape hatch for
`todo!()` after `no_panic` becomes a true tool lint.

Because `#[allow(tigerstyle::no_panic)]` suppresses the `no_panic` lint at the
rustc level, the system MUST audit that escape hatch with a companion lint,
`unjustified_no_panic_allow`, so misuse still warns.

The system MUST NOT fire either lint for `todo!()` when the call is inside a
function or block explicitly allowed with `#[allow(tigerstyle::no_panic)]` and
an adjacent justification comment.

For this rule, an adjacent justification comment is a `// ...` comment on the
same line as the allow attribute or on the immediately preceding non-empty
line. Blank lines, doc comments, and block comments do NOT count.

The system MUST warn via `unjustified_no_panic_allow` when any of these hold:
- `#[allow(tigerstyle::no_panic)]` appears without that adjacent `//`
  justification comment
- `#[allow(tigerstyle::no_panic)]` is used around `panic!()`
- `#[allow(tigerstyle::no_panic)]` is used around `unimplemented!()`
- `#[allow(tigerstyle::no_panic)]` is used around `unreachable!()`

#### Scenario: Justified todo allow

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Justified block todo allow

- GIVEN a block annotated with `#[allow(tigerstyle::no_panic)]` and an
  adjacent `//` justification comment that contains `todo!(...)`
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unjustified todo still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` without that
  adjacent `//` justification comment
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Panic escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { panic!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unimplemented escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { unimplemented!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unreachable escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { unreachable!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic
