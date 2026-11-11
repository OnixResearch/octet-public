## ADDED Requirements

### Requirement: Performance Guide Discoverability

The public documentation set MUST expose the Tiger Style performance guide and
copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Performance review guide](docs/performance-review.md)`
- `[Performance sketch template](templates/performance-sketch.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with exact
text and repo-valid destinations:

- `[Performance review guide](performance-review.md)`
- `[Performance sketch template](../templates/performance-sketch.md)`
- `[Performance spec contract](../openspec/specs/performance/spec.md)`


#### Scenario: Reader finds the performance guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Performance review guide](docs/performance-review.md)` and
  `[Performance sketch template](templates/performance-sketch.md)` without
  reading archived OpenSpec changes or private skill files

#### Scenario: Contributor sees performance doc touchpoints

- GIVEN a contributor changes public Tiger Style guidance for performance review
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Performance review guide](performance-review.md)`,
  `[Performance sketch template](../templates/performance-sketch.md)`, and
  `[Performance spec contract](../openspec/specs/performance/spec.md)` in the
  maintained documentation crosswalk
