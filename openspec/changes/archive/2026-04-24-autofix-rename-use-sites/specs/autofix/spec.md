## MODIFIED Requirements

### Requirement: Initial Autofix Suggestion Coverage

The existing applicability tier contract for rename-oriented suggestions gains
a use-site rewriting clause:

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

### Requirement: Autofix Validation Artifacts

The proof contract gains additional obligations for the new `MachineApplicable`
patterns:

- `.fixed` fixture coverage MUST include at least one `bool_naming`
  local-variable rename with multiple use sites
- `.fixed` fixture coverage MUST include at least one `negated_predicate`
  local-variable rename with multiple use sites
- the saved `cargo fix` transcript MUST exercise at least one
  local-variable rename from each promoted lint

#### Scenario: Review checks local-variable rename proof

- GIVEN a maintainer reviews a `bool_naming` or `negated_predicate`
  local-variable pattern marked `MachineApplicable`
- WHEN they inspect the saved autofix validation artifacts
- THEN they find a `.fixed` fixture showing the post-fix code with declaration
  and all use sites renamed
- AND the `.fixed` fixture compiles cleanly through the `autofix_machine_applicable`
  test rail
