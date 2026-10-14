## ADDED Requirements

### Requirement: Rollout Kit Records New-Lint Aspen Classification

The downstream rollout kit MUST record Aspen evidence-backed classification for
newly added lint families before any starter-profile promotion.

The guidance MUST include:

- the per-lint evidence artifacts used for Aspen validation
- the classification of each pending lint family as `starter-candidate` or
  `inventory-first`
- the rule that inventory-first families stay disabled in starter-profile
  examples until a later promotion review lands
- any new lint-family names that consumers need to recognize during rollout

#### Scenario: Maintainer evaluates a newly added lint family

- GIVEN tigerstyle adds new warn-by-default lint families that are not yet in a
  starter rollout profile
- WHEN a maintainer follows the rollout kit
- THEN they can find the saved Aspen evidence, see the current classification
  for each family, and keep inventory-first families out of copied starter
  configs until later promotion work is complete
