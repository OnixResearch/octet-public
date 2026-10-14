# Documentation Specification — Delta Spec

## ADDED Requirements

### Requirement: Downstream Consumer Rollout Plan

The project MUST maintain a documented rollout plan for integrating
`tigerstyle` into a real consumer workspace before claiming downstream
readiness.

The rollout plan MUST:

- name the target workspace
- identify the initial lint subset enabled for that workspace
- explain how the workspace wires tigerstyle into flake checks or equivalent
  CI
- record which lint families are deferred because of known noise or missing
  project-specific tuning
- define promotion criteria for moving a deferred lint family into a later
  rollout phase

#### Scenario: Preparing the first aspen rollout

- GIVEN aspen is the first downstream consumer
- WHEN a maintainer follows the rollout plan
- THEN aspen can run a scoped tigerstyle check without blindly enabling the
  full lint catalog

### Requirement: Consumer Baseline And Configuration Evidence

The project MUST capture baseline evidence and required configuration
adjustments for each downstream workspace onboarded through the rollout plan.

The integration notes MUST record:

- baseline violation counts or representative failures from the initial run
- required `dylint.toml` thresholds or allowlists
- rationale for each committed threshold, allowlist, or explicit exception used
  by the rollout
- validation steps showing the configured check runs end to end in the
  consumer workspace

#### Scenario: Reviewing a tuned aspen configuration

- GIVEN aspen needs workspace-specific thresholds or lint allowlists
- WHEN a maintainer reviews the integration notes
- THEN the reviewer can see why the config exists and how the check was
  validated
