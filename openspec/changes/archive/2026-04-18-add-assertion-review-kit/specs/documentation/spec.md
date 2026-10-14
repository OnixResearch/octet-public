## ADDED Requirements

### Requirement: Assertion Guide Discoverability

The public documentation set MUST expose the Tiger Style assertion guide and
copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Assertion review guide](docs/assertion-review.md)`
- `[Assertion checklist template](templates/assertion-checklist.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Assertion review guide](assertion-review.md)`
- `[Assertion checklist template](../templates/assertion-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

#### Scenario: Reader finds the assertion guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Assertion review guide](docs/assertion-review.md)` and
  `[Assertion checklist template](templates/assertion-checklist.md)`

#### Scenario: Contributor sees assertion doc touchpoints

- GIVEN a contributor changes public Tiger Style assertion guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Assertion review guide](assertion-review.md)`,
  `[Assertion checklist template](../templates/assertion-checklist.md)`, and
  `[Safety spec contract](../openspec/specs/safety/spec.md)`
