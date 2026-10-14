# Documentation — Delta Spec

## MODIFIED Requirements

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
- documented minimum environment assumptions that match the published template's
  actual runtime dependencies

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

## ADDED Requirements

### Requirement: Audit-Ready Rollout Reconstruction Evidence

When the project claims a reusable downstream rollout kit is complete, it MUST
capture audit-ready evidence showing the documented reconstruction path works.

The evidence MUST include:

- a real end-to-end reconstruction transcript using the published templates or
  equivalent copied artifacts, not only syntax or help output
- the full helper scripts used for validation, stored as reviewable files
- verification notes that reference exact commands or stored helper scripts
  instead of elided here-doc summaries
- enough detail to show the documented starter path was reconstructed without
  relying on undocumented shell state or current downstream repo drift

#### Scenario: Reviewing Aspen phase-1 reconstruction

- GIVEN a reviewer inspects the rollout-kit verification artifacts
- WHEN they open the reconstruction evidence
- THEN they can see the exact helper scripts, the temp-worktree setup, and the
  transcript proving the published Aspen phase-1 path ran from the new templates
