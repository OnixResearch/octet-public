## ADDED Requirements

### Requirement: No Panic Allow Hygiene

The system MUST warn when `#[allow(tigerstyle::no_panic)]` remains on an item or
block that contains no `todo!()`, `panic!()`, `unimplemented!()`, or
`unreachable!()` call covered by that escape hatch.

This warning MUST use `unjustified_no_panic_allow` so the existing companion
lint remains the single audit path for `no_panic` escapes.

#### Scenario: Live justified todo allow stays quiet

- GIVEN `// justified stub while API lands`
  `#[allow(tigerstyle::no_panic)] fn helper() { todo!("later"); }`
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Orphaned item allow

- GIVEN `#[allow(tigerstyle::no_panic)] fn helper() { do_work(); }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Orphaned block allow

- GIVEN a block annotated with `#[allow(tigerstyle::no_panic)]` whose body has
  no panic-family macro
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: `cfg_attr`-expanded orphan allow

- GIVEN `cfg_attr(...)` expands into a direct `#[allow(tigerstyle::no_panic)]`
  on a supported item or block, and that annotated body has no panic-family
  macro
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic
