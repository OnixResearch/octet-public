## ADDED Requirements

### Requirement: Safety-Critical Guide Discoverability

The public documentation set MUST expose the safety-critical claims guide from
its documented entry points.

`README.md` MUST contain this literal Markdown link with exact text and
repo-valid destination:

- `[Safety-critical claims guide](docs/safety-critical.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Safety-critical claims guide](safety-critical.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

#### Scenario: Reader finds the safety-critical guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for public Tiger Style guidance beyond lint catalogs
- THEN they can find the exact Markdown link
  `[Safety-critical claims guide](docs/safety-critical.md)`

#### Scenario: Contributor sees safety-critical doc touchpoints

- GIVEN a contributor changes public safety-critical guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Safety-critical claims guide](safety-critical.md)` and
  `[Safety spec contract](../openspec/specs/safety/spec.md)`
