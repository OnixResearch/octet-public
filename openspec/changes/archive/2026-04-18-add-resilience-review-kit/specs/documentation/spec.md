## ADDED Requirements

### Requirement: Resilience Guide Discoverability

The public documentation set MUST expose the resilience review guide and
five-whys template from its documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Resilience review guide](docs/resilience-review.md)`
- `[Five-whys architecture template](templates/five-whys-architecture.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Resilience review guide](resilience-review.md)`
- `[Five-whys architecture template](../templates/five-whys-architecture.md)`

#### Scenario: Reader finds the resilience guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style guidance beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Resilience review guide](docs/resilience-review.md)` and
  `[Five-whys architecture template](templates/five-whys-architecture.md)`

#### Scenario: Contributor sees resilience doc touchpoints

- GIVEN a contributor changes public resilience guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Resilience review guide](resilience-review.md)` and
  `[Five-whys architecture template](../templates/five-whys-architecture.md)`
