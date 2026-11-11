## ADDED Requirements

### Requirement: Rollout Kit Has A Saved Non-Aspen Consumer Proof

The downstream rollout kit MUST include one saved proof that a non-Aspen Rust
consumer can follow the published templates and check-path guidance.

That proof MUST include:

- the chosen consumer workspace
- the copied `templates/tigerstyle-check.sh` and `templates/dylint.toml`
  workflow, or an exact saved helper that performs those copy/edit steps
- a saved transcript of the resulting tigerstyle check run
- enough sandbox or path-layout detail for a reviewer to replay the proof
  without Aspen-specific assumptions

#### Scenario: Reviewing the second consumer proof

- GIVEN a reviewer wants to know whether the rollout kit only works for Aspen
- WHEN they inspect the saved proof
- THEN they can replay the helper on a non-Aspen consumer and see the template-
  based tigerstyle check run end to end
