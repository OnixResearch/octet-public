## ADDED Requirements

### Requirement: Rollout Kit Covers Post-Starter Package Widening

The downstream rollout kit MUST document how a consumer graduates a deferred
package into the default tigerstyle path after the starter rollout is green.

The guidance MUST include:

- the conditions for choosing one deferred package as the next promotion target
- the cleanup themes reviewers should classify before widening the default path
- a concrete widened-reference state derived from Aspen after `aspen-coordination`
  rejoined the default pilot
- evidence showing both the widened default pilot and the cleaned deferred
  package slice

#### Scenario: Promoting a deferred package after phase 1

- GIVEN a consumer finished the starter rollout and still has one deferred
  package with a bounded finding set
- WHEN a maintainer follows the rollout kit
- THEN they can inventory the slice, clean or justify the blockers, and prove
  the package can join the default path without rediscovering Aspen's phase-2
  promotion steps
