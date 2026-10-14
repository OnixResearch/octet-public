# Documentation — Delta Spec

## ADDED Requirements

### Requirement: Reusable Consumer Integration Kit

After the first real downstream rollout, the project MUST generalize the
consumer-specific lessons into a reusable onboarding kit for later consumers.

The integration kit MUST include:

- named starter rollout profiles with rationale for why they are suitable as
  first adopters
- a reusable check-path recipe for consumers whose toolchain or environment
  needs Dylint wrappers or extra setup
- a sample `dylint.toml` layout with commented threshold, allowlist, and
  rationale sections
- a checklist for deciding when a new crate or package is ready to join the
  default pilot scope
- pointers to evidence from the first consumer rollout that justify the kit's
  defaults and cautions

#### Scenario: Preparing a second downstream rollout

- GIVEN a maintainer is onboarding a second consumer after Aspen
- WHEN they follow the integration kit
- THEN they can choose a starter profile, wire a working check path, and add
  consumer-local tuning without rediscovering Aspen's setup work

#### Scenario: Reviewing a consumer wrapper

- GIVEN a consumer uses a nix-managed or otherwise nonstandard nightly toolchain
- WHEN a reviewer checks the documented integration kit
- THEN they can see the expected wrapper responsibilities and minimum
  environment assumptions

### Requirement: High-Noise Lint Promotion Playbook

The project MUST maintain a promotion playbook for lint families that proved
too noisy for blind rollout in Aspen.

The playbook MUST:

- name `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, and
  `unbounded_loop` as inventory-first families
- require a per-crate or per-package inventory before a family is promoted
  into the default downstream path
- record known noise or classification patterns that reviewers should sort
  before deciding whether a finding is real debt, justified shell code, or a
  lint-precision problem
- define the evidence required before promoting a family into a later rollout
  phase
- reject blanket `allow`-based promotion as a substitute for triage

#### Scenario: Promoting `no_unwrap`

- GIVEN a maintainer wants to add `no_unwrap` to a downstream default profile
- WHEN they follow the playbook
- THEN they first build an inventory, classify the highest-risk production
  sites, and capture evidence for the reduced baseline before promotion

#### Scenario: Reviewing `unbounded_loop` findings

- GIVEN a downstream crate has many loop-related findings
- WHEN a reviewer follows the playbook
- THEN they can separate real unbounded retries from known event-loop patterns
  and identify any remaining lint-precision gaps explicitly
