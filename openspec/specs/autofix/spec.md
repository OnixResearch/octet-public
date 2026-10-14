# autofix Specification

## Purpose

Define tigerstyle's structured suggestion contract, rollout scope, and proof
obligations for reviewable autofix support.

## Requirements
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
- AND rerunning the same automatic fix leaves the validated post-fix fixture unchanged
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
- `negated_predicate`
- `float_for_currency`
- `no_todo`
- `ambient_clock`
- `no_unwrap`
- `sentinel_fallback`
- `ignored_result`
- `contradictory_time`
- `unbounded_channel`
- `explicit_defaults`
- `catch_all_on_enum`
- `usize_in_public_api`
- `multi_lock_ordering`
- `compound_condition`
- `unbounded_collection_growth`

For this rollout, tigerstyle MUST keep the suggestion tier honest for each
reported pattern:

- `compound_assertion` MUST only use `MachineApplicable` for cases whose
  emitted rewrite is semantics-preserving for the validated pattern; top-level
  `assert!(a || b)` diagnostics MUST NOT claim `MachineApplicable` unless the
  project first proves a semantics-preserving rewrite for that pattern
- rename-oriented suggestions such as `bool_naming`, `acronym_style`, and
  `negated_predicate` MUST NOT claim `MachineApplicable` when the emitted edit
  rewrites only the declaration span; declaration-only rename suggestions MUST
  use `MaybeIncorrect` or `HasPlaceholders` unless the implementation also
  rewrites the affected use sites needed to keep the validated fixture compiling
- `bool_naming` and `negated_predicate` MAY use `MachineApplicable` for
  local-variable rename patterns (triggered by `check_local`) when the
  implementation collects all references to the flagged binding within the
  enclosing function body and the emitted multipart suggestion covers
  declaration plus every collected use site
- the `MachineApplicable` promotion for local-variable renames MUST fall back
  to `MaybeIncorrect` when any collected span originates from macro expansion
  or when the binding escapes the enclosing function body through a closure
  capture that the lint cannot rewrite
- `numeric_units`, `unjustified_allow`, `float_for_currency`, `no_todo`,
  `ambient_clock`, `unbounded_channel`, `explicit_defaults`, `catch_all_on_enum`,
  `multi_lock_ordering`, `compound_condition`, and `unbounded_collection_growth`
  MUST use `HasPlaceholders` when the emitted text still requires the maintainer
  to choose a value, fill in a justification, or complete a placeholder
- `unchecked_division`, `raw_arithmetic_overflow`, `unchecked_narrowing`,
  `platform_dependent_cast`, `no_unwrap`, `sentinel_fallback`, `ignored_result`,
  `contradictory_time`, and `usize_in_public_api` MUST use `MaybeIncorrect`
  when the suggested change may alter guards, control flow, error propagation,
  or type-width behavior

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

#### Scenario: Local bool binding rename covers use sites

- GIVEN `bool_naming` reports a local `let ready = true;` binding
- WHEN tigerstyle emits the structured suggestion for that pattern
- THEN the suggestion is a multipart suggestion covering the declaration span
  and every use of `ready` within the enclosing function body
- AND the suggestion uses `MachineApplicable`
- AND the `.fixed` proof compiles without manual edits

#### Scenario: Local negated predicate binding rename covers use sites

- GIVEN `negated_predicate` reports a local `let is_not_ready = false;` binding
- WHEN tigerstyle emits the structured suggestion for that pattern
- THEN the suggestion is a multipart suggestion covering the declaration span
  and every use of `is_not_ready` within the enclosing function body
- AND the suggestion uses `MachineApplicable`
- AND the `.fixed` proof compiles without manual edits

#### Scenario: Macro-expanded binding falls back to review-required

- GIVEN `bool_naming` reports a local binding whose declaration or a use site
  has a span from macro expansion
- WHEN tigerstyle emits the structured suggestion
- THEN the suggestion uses `MaybeIncorrect` instead of `MachineApplicable`
- AND the diagnostic still shows the suggested rename for manual review

#### Scenario: Function-level negated predicate stays review-required

- GIVEN `negated_predicate` reports a function or method name (not a local
  binding)
- WHEN tigerstyle emits the structured suggestion
- THEN the suggestion still uses `HasPlaceholders` or `MaybeIncorrect`
- AND the function-level rename is not promoted to `MachineApplicable`

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
- `no_recursion`
- `nested_conditionals`
- `no_panic`
- `too_many_parameters`
- `unbounded_loop`
- `unjustified_no_todo_allow`
- `verified_purity`

Note: `unjustified_no_todo_allow` shares its source file with `no_todo`
(which is in the rollout). The rollout scope test exempts shared-source lints
from the "source must not contain emit_tiger_style_lint_with_suggestion"
assertion.

#### Scenario: Diagnostics-only lint stays outside autofix scope

- GIVEN a maintainer reviews `assertion_density` or another diagnostics-only
  lint named above
- WHEN they inspect this change's acceptance scope
- THEN they do not expect structured suggestions or autofix proof artifacts for
  that lint in this change

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
- `.fixed` fixture coverage MUST include at least one `bool_naming`
  local-variable rename with multiple use sites
- `.fixed` fixture coverage MUST include at least one `negated_predicate`
  local-variable rename with multiple use sites
- the saved `cargo fix` transcript MUST exercise at least one
  local-variable rename from each promoted lint

Tigerstyle MUST NOT promote a suggestion pattern to `MachineApplicable`
without the matching `.fixed` proof for that pattern.

#### Scenario: Review checks machine-applicable proof

- GIVEN a maintainer reviews a lint pattern marked `MachineApplicable`
- WHEN they inspect the saved autofix validation artifacts
- THEN they can find the matching `.fixed` proof for that pattern
- AND the proof shows the post-fix code compiling cleanly

#### Scenario: Review checks local-variable rename proof

- GIVEN a maintainer reviews a `bool_naming` or `negated_predicate`
  local-variable pattern marked `MachineApplicable`
- WHEN they inspect the saved autofix validation artifacts
- THEN they find a `.fixed` fixture showing the post-fix code with declaration
  and all use sites renamed
- AND the `.fixed` fixture remains unchanged on a second `cargo fix` pass through
  the `autofix_machine_applicable` test rail
- AND the `.fixed` fixture compiles cleanly through the same test rail

#### Scenario: Review checks manual-only suggestion proof

- GIVEN a maintainer reviews a lint pattern marked `MaybeIncorrect` or
  `HasPlaceholders`
- WHEN they inspect the saved UI artifacts
- THEN they can see the exact suggestion text in the `.stderr` snapshot
- AND the change does not claim automatic application proof for that pattern

