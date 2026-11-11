## MODIFIED Requirements

### Requirement: Panic Macro In Production Code

The system MUST detect `panic!()`, `unimplemented!()`, and `unreachable!()` in
production code through `no_panic`.

The system MUST detect `todo!()` in production code through a dedicated safety
lint, `no_todo`.

The system MUST register `no_todo` as a warn-by-default safety tool lint in
canonical `tigerstyle::no_todo` form, with the repo's current one-member bare
alias path continuing to resolve supported bare source/config spellings.

The system MUST keep the repo's supported test-context suppressions for both
lints, including `#[cfg(test)]`, `#[test]`, and the shared file/context
heuristics already used by tigerstyle.

The system MUST NOT treat `#[allow(tigerstyle::no_panic)]` or
`#[allow(no_panic)]` as the documented escape hatch for `todo!()` once the
split lands.

#### Scenario: `todo!()` emits the dedicated lint

- GIVEN production code that invokes `todo!(...)`
- WHEN the lint suite runs after the split
- THEN `no_todo` emits the diagnostic
- AND `no_panic` does not claim that same `todo!()` call

#### Scenario: panic-family macros stay on `no_panic`

- GIVEN production code that invokes `panic!(...)`, `unimplemented!(...)`, or
  `unreachable!(...)`
- WHEN the lint suite runs after the split
- THEN `no_panic` emits the diagnostic
- AND `no_todo` does not claim those macros

#### Scenario: legacy tool-scoped `no_panic` allow does not hide `todo!()`

- GIVEN a `todo!(...)` stub under `#[allow(tigerstyle::no_panic)]`
- WHEN the lint suite runs after the split
- THEN `no_todo` still emits the diagnostic
- AND the maintainer must migrate that escape hatch to `no_todo`

#### Scenario: legacy bare `no_panic` allow does not hide `todo!()`

- GIVEN a `todo!(...)` stub under `#[allow(no_panic)]`
- WHEN the lint suite runs after the split
- THEN `no_todo` still emits the diagnostic
- AND the maintainer must migrate that escape hatch to `no_todo`

## REMOVED Requirements

### Requirement: No Panic Allow Hygiene

The temporary-stub escape hatch no longer hangs off `no_panic`; this requirement
is replaced by the dedicated `no_todo` allow-audit contract below.

## ADDED Requirements

### Requirement: No Todo Allow Hygiene

The system MUST support a narrow temporary-stub escape hatch for `todo!()` via
`#[allow(tigerstyle::no_todo)]` and the supported bare compatibility spelling
`#[allow(no_todo)]`.

The system MUST register `unjustified_no_todo_allow` as a warn-by-default
safety lint in canonical tool-scoped form, while the repo's current bare alias
path continues to resolve the supported bare source/config spellings.

The system MUST NOT fire either `no_todo` or `unjustified_no_todo_allow` when a
`todo!()` call sits inside a function or block explicitly allowed with either
`#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` and an adjacent
justification comment.

For this rule, an adjacent justification comment is a `// ...` comment on the
same line as the allow attribute or on the immediately preceding non-empty
line. Blank lines, doc comments, and block comments do NOT count.

The system MUST warn via `unjustified_no_todo_allow` when any of these hold:
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` appears without that
  adjacent `//` justification comment
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` appears around
  `panic!()`, `unimplemented!()`, or `unreachable!()` instead of a `todo!()`
  stub
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` remains on an item or
  block after the covered `todo!()` stub is gone
- `cfg_attr(...)` expands into one of those supported direct allow attrs on a
  covered item or block, and the resulting expanded attr is unjustified or
  orphaned

#### Scenario: justified tool-scoped `no_todo` allow stays quiet

- GIVEN `#[allow(tigerstyle::no_todo)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_todo` nor `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: justified bare `no_todo` allow stays quiet

- GIVEN `#[allow(no_todo)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_todo` nor `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: unjustified `no_todo` allow warns

- GIVEN `#[allow(tigerstyle::no_todo)] fn stub() { todo!(...) }` without that
  adjacent `//` justification comment
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: orphaned or misapplied `no_todo` allow warns

- GIVEN `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` either wraps a
  non-`todo!()` panic-family macro or covers no remaining `todo!()` call at all
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: `cfg_attr`-expanded `no_todo` allow is audited

- GIVEN `cfg_attr(...)` expands into a direct `#[allow(tigerstyle::no_todo)]`
  or `#[allow(no_todo)]` on a supported item or block
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` applies the same justification and orphan
  checks to that expanded attr
