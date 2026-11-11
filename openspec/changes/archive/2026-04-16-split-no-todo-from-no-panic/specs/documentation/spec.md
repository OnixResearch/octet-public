## MODIFIED Requirements

### Requirement: Tool-Lint Migration Precondition

Before converting all current consumer-facing lints, the project MUST validate
the planned migration mechanism on a single representative lint under the
repo's Dylint harness.

That precondition MUST prove both of these points together:

- `declare_tool_lint!(pub tigerstyle::...)` works for the representative lint
  under the project's Dylint loading path
- the intended bare compatibility mechanism — a one-member bare alias
  registered through `lint_store.register_group(...)` — still resolves the
  current bare config / lint-level path for that representative lint

#### Scenario: Representative spike still proves canonical and bare paths

- GIVEN representative lint `no_unwrap` has already proven the tool-lint spike
  under the repo's Dylint harness
- WHEN tigerstyle adds `no_todo` as another true tool lint with bare alias
  compatibility
- THEN the repo still follows the same validated migration pattern instead of
  inventing a second source-attr/config bridge

## ADDED Requirements

### Requirement: Todo Escape Migration Guidance

The documentation set MUST identify `no_todo` as the supported
temporary-stub lint after the split in `README.md`,
`docs/downstream-rollout.md`, and the starter-template comments in
`templates/dylint.toml`.

That guidance MUST tell maintainers to migrate existing justified
`#[allow(tigerstyle::no_panic)]` and `#[allow(no_panic)]` `todo!()` escapes to
`#[allow(tigerstyle::no_todo)]` or the supported bare form `#[allow(no_todo)]`.

That guidance MUST also name the companion audit lint
`unjustified_no_todo_allow`, explain that `no_panic` now covers only
`panic!()`, `unimplemented!()`, and `unreachable!()`, and carry an explicit
breaking-change callout where the old `no_panic` stub escape hatch used to be
documented.

#### Scenario: Maintainer sees the `no_todo` migration path

- GIVEN a downstream workspace that already carries justified `todo!()` stubs
  under `allow(no_panic)` or `allow(tigerstyle::no_panic)`
- WHEN a maintainer follows the repo's rollout docs after this change
- THEN they see the required rename to `no_todo`
- AND they see the narrowed post-split meaning of `no_panic`
- AND they see an explicit breaking-change callout for the old stub escape
  hatch spelling
