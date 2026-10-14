## ADDED Requirements

### Requirement: Deref Polymorphism Lint [r[deref-polymorphism-lint]]

The structure lint inventory MUST include an advisory `deref_polymorphism` lint that identifies public `Deref` implementations which make one public local type inherit a nominal target type's API surface.

#### Scenario: Public Deref to nominal target warns [r[deref-polymorphism-lint.warn-public-nominal-target]]

- GIVEN a public local wrapper type implements `Deref<Target = Engine>`
- WHEN Tigerstyle analyzes the crate
- THEN `tigerstyle::deref_polymorphism` MUST warn that public `Deref` to a nominal target hides API boundaries and should prefer explicit accessors

#### Scenario: Accepted Deref targets do not warn [r[deref-polymorphism-lint.accepted-targets]]

- GIVEN private wrappers, non-`Deref` impls, `Deref<Target = str>`, or `Deref<Target = [T]>`
- WHEN Tigerstyle analyzes the crate
- THEN `tigerstyle::deref_polymorphism` MUST NOT warn for those accepted borrowed-view or non-public cases

#### Scenario: Inventory and rollout metadata stay synchronized [r[deref-polymorphism-lint.inventory]]

- GIVEN the lint is implemented
- WHEN docs, templates, diagnostics, and fixture manifests are checked
- THEN the lint catalog MUST list `deref_polymorphism` as a default-warn structure lint and rollout examples MUST keep it advisory until future promotion evidence exists
