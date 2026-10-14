## MODIFIED Requirements

### Requirement: Guarded unwraps remain precise

The system MUST suppress `no_unwrap` only when an unwrap is dominated by a same-branch `.is_some()` or `.is_ok()` guard and MUST keep warning when the unwrap is unguarded or guarded only through a disjunction.

#### Scenario: Same-branch guard suppression

- GIVEN an `if value.is_some()` branch
- WHEN the same branch unwraps `value`
- THEN `no_unwrap` does not warn

#### Scenario: Disjunctive guard still warns

- GIVEN an `if value.is_some() || other_condition` branch
- WHEN that branch unwraps `value`
- THEN `no_unwrap` still warns
