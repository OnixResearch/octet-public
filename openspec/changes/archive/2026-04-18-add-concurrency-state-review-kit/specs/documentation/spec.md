## ADDED Requirements

### Requirement: Concurrency Guide Discoverability

The public documentation set MUST expose the Tiger Style concurrency and
 distributed-state guide and copyable template from the repo's documented
 entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)`
- `[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Concurrency and distributed-state review guide](concurrency-state-review.md)`
- `[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)`
- `[Concurrency spec contract](../openspec/specs/concurrency/spec.md)`

#### Scenario: Reader finds the concurrency guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)`
  and `[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)`

#### Scenario: Contributor sees concurrency doc touchpoints

- GIVEN a contributor changes public Tiger Style concurrency guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Concurrency and distributed-state review guide](concurrency-state-review.md)`,
  `[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)`,
  and `[Concurrency spec contract](../openspec/specs/concurrency/spec.md)`
