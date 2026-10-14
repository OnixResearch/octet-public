## ADDED Requirements

### Requirement: System Comprehension Guide Discoverability

The public documentation set MUST expose the system-comprehension guide and
walkthrough template from its documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[System comprehension guide](docs/system-comprehension.md)`
- `[System walkthrough template](templates/system-walkthrough.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[System comprehension guide](system-comprehension.md)`
- `[System walkthrough template](../templates/system-walkthrough.md)`

#### Scenario: Reader finds the system comprehension guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style guidance beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[System comprehension guide](docs/system-comprehension.md)` and
  `[System walkthrough template](templates/system-walkthrough.md)`

#### Scenario: Contributor sees system-comprehension doc touchpoints

- GIVEN a contributor changes public system-comprehension guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[System comprehension guide](system-comprehension.md)` and
  `[System walkthrough template](../templates/system-walkthrough.md)`
