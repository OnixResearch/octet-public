## ADDED Requirements

### Requirement: Bounds And Pacing Guide Discoverability

The public documentation set MUST expose the Tiger Style bounds-and-pacing
guide and copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)`
- `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Bounds and pacing review guide](bounds-and-pacing-review.md)`
- `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

#### Scenario: Reader finds the bounds guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)` and
  `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`

#### Scenario: Contributor sees bounds doc touchpoints

- GIVEN a contributor changes public Tiger Style boundedness guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Bounds and pacing review guide](bounds-and-pacing-review.md)`,
  `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`,
  and `[Safety spec contract](../openspec/specs/safety/spec.md)`
