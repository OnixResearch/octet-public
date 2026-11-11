## ADDED Requirements

### Requirement: Structured Suggestion Applicability Contract

Tigerstyle MUST express autofix-capable diagnostics through rustc structured
suggestions such as `span_suggestion` or `multipart_suggestion`, not only
free-text help text.

Every structured suggestion MUST declare an explicit applicability tier:

- `MachineApplicable` only when applying the emitted replacement preserves the
  intended meaning for the validated case and leaves the checked fixture
  compiling without manual edits
- `MaybeIncorrect` when the suggestion is concrete but may change control flow,
  error propagation, or the wider rename/update scope and therefore needs human
  review
- `HasPlaceholders` when the replacement text intentionally contains a stub,
  placeholder, or human choice that must be filled in before the code is ready

`cargo fix` eligibility in tigerstyle MUST follow that applicability contract:
`MachineApplicable` suggestions are the only tier this change may require to be
applied automatically.

For this change, the supported automatic-application proof path MUST be a
saved `cargo fix` run inside the repo's nix devshell. `rustfix` is not a
second required acceptance path.

#### Scenario: Machine-applicable suggestion is eligible for cargo fix

- GIVEN a lint emits a `MachineApplicable` structured suggestion
- WHEN a maintainer validates that case through the saved nix-devshell
  `cargo fix` proof path
- THEN the emitted replacement is eligible for automatic application
- AND the validated post-fix fixture still compiles without manual edits

#### Scenario: Review-required suggestion stays manual

- GIVEN a lint emits a `MaybeIncorrect` or `HasPlaceholders` suggestion
- WHEN a maintainer runs tigerstyle on violating code
- THEN the diagnostic still shows the suggested replacement
- AND the maintainer must review or complete that change manually instead of
  relying on automatic application

### Requirement: Initial Autofix Suggestion Coverage

Tigerstyle MUST add structured suggestions for the initial autofix rollout in
this change.

The rollout MUST cover these lint families:

- `compound_assertion`
- `bool_naming`
- `acronym_style`
- `numeric_units`
- `unchecked_division`
- `raw_arithmetic_overflow`
- `unchecked_narrowing`
- `platform_dependent_cast`
- `unjustified_allow`

For this rollout, tigerstyle MUST keep the suggestion tier honest for each
reported pattern:

- `compound_assertion` MUST only use `MachineApplicable` for cases whose
  emitted rewrite is semantics-preserving for the validated pattern; top-level
  `assert!(a || b)` diagnostics MUST NOT claim `MachineApplicable` unless the
  project first proves a semantics-preserving rewrite for that pattern
- rename-oriented suggestions such as `bool_naming` and `acronym_style` MUST
  NOT claim `MachineApplicable` when the emitted edit rewrites only the
  declaration span; declaration-only rename suggestions MUST use
  `MaybeIncorrect` unless the implementation also rewrites the affected use
  sites needed to keep the validated fixture compiling
- `numeric_units` and `unjustified_allow` MUST use `HasPlaceholders` when the
  emitted text still requires the maintainer to choose a unit or fill in a
  real justification
- `unchecked_division`, `raw_arithmetic_overflow`, `unchecked_narrowing`, and
  `platform_dependent_cast` MUST use `MaybeIncorrect` when the suggested change
  may alter guards, control flow, or conversion/error-handling behavior

#### Scenario: Top-level conjunction gets machine-applicable split

- GIVEN `compound_assertion` reports a top-level `assert!(a && b)`
- WHEN tigerstyle emits the structured suggestion for that validated pattern
- THEN the suggestion may use `MachineApplicable`
- AND the replacement expands into separate assertions that keep the validated
  fixture compiling

#### Scenario: Top-level disjunction does not overclaim autofix safety

- GIVEN `compound_assertion` reports a top-level `assert!(a || b)`
- WHEN tigerstyle emits a diagnostic for that pattern
- THEN the diagnostic does not label the rewrite `MachineApplicable` unless a
  semantics-preserving rewrite has been proven for that exact pattern

#### Scenario: Declaration-only rename stays review-required

- GIVEN `bool_naming` or `acronym_style` reports a declaration whose emitted
  edit renames only the declaration span
- WHEN tigerstyle emits the structured suggestion
- THEN that suggestion uses `MaybeIncorrect`
- AND the maintainer is signaled that wider rename propagation still needs
  review

#### Scenario: Placeholder suggestion exposes missing human choice

- GIVEN `numeric_units` or `unjustified_allow` reports code that still needs a
  maintainer-selected unit or justification
- WHEN tigerstyle emits the structured suggestion
- THEN the suggestion uses `HasPlaceholders`
- AND the replacement text makes the missing human choice explicit

### Requirement: Out-Of-Rollout Lints Stay Explicitly Out Of Scope

This change MUST leave these lint families diagnostics-only, without structured
suggestions or autofix proof artifacts:

- `assertion_density`
- `function_length`
- `ambiguous_params`
- `multi_lock_ordering`
- `catch_all_on_enum`
- `no_recursion`

This change MUST also leave these additional shipped lint families unchanged
and out of scope for structured suggestions:

- `ambient_clock`
- `compound_condition`
- `contradictory_time`
- `explicit_defaults`
- `float_for_currency`
- `ignored_result`
- `negated_predicate`
- `nested_conditionals`
- `no_panic`
- `no_unwrap`
- `too_many_parameters`
- `unbounded_channel`
- `unbounded_collection_growth`
- `unbounded_loop`
- `unjustified_no_panic_allow`
- `usize_in_public_api`
- `verified_purity`

#### Scenario: Diagnostics-only lint stays outside autofix scope

- GIVEN a maintainer reviews `assertion_density` or another diagnostics-only
  lint named above
- WHEN they inspect this change's acceptance scope
- THEN they do not expect structured suggestions or autofix proof artifacts for
  that lint in this change

#### Scenario: Unchanged lint stays outside rollout work

- GIVEN a maintainer reviews `ambient_clock` or another unchanged lint named
  above
- WHEN they inspect this change's acceptance scope
- THEN they do not expect structured suggestions, autofix artifacts, or task
  work for that lint in this change

### Requirement: Autofix Validation Artifacts

Tigerstyle MUST prove structured suggestion behavior with deterministic review
artifacts.

The proof contract for this change MUST include:

- ignored UI coverage that records suggestion text in `.stderr` snapshots for
  every lint in the initial autofix rollout
- a saved ignored UI artifact that prints each exercised autofix fixture name
  and makes the covered snapshot set explicit for deterministic review
- `.fixed` fixture coverage for every `MachineApplicable` suggestion pattern
  exercised by this change
- a saved nix-devshell `cargo fix` transcript showing the exact proof command
  and the machine-applicable target it exercised

Tigerstyle MUST NOT promote a suggestion pattern to `MachineApplicable`
without the matching `.fixed` proof for that pattern.

#### Scenario: Review checks machine-applicable proof

- GIVEN a maintainer reviews a lint pattern marked `MachineApplicable`
- WHEN they inspect the saved autofix validation artifacts
- THEN they can find the matching `.fixed` proof for that pattern
- AND the proof shows the post-fix code compiling cleanly

#### Scenario: Review checks manual-only suggestion proof

- GIVEN a maintainer reviews a lint pattern marked `MaybeIncorrect` or
  `HasPlaceholders`
- WHEN they inspect the saved UI artifacts
- THEN they can see the exact suggestion text in the `.stderr` snapshot
- AND the change does not claim automatic application proof for that pattern
