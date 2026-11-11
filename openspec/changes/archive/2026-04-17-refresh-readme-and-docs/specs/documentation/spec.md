## ADDED Requirements

### Requirement: README Landing Page

The repo MUST keep `README.md` as a newcomer landing page instead of the only
full manual.

That landing page MUST, near the top of the file:

- state what tigerstyle is
- show the first-class consumer run path with `cargo tigerstyle check` and the
  matching `nix run ...#cargo-tigerstyle -- check` form
- link to deeper docs by audience instead of embedding every rollout detail in
  the README itself
- summarize the current naming surface well enough that a new reader can tell
  the project name, binary name, command name, and lint/config namespaces apart

#### Scenario: New visitor finds the first-class run path from README

- GIVEN a new reader opens `README.md`
- WHEN they look for the supported way to run tigerstyle in a consumer workspace
- THEN they can find `cargo tigerstyle check`
- AND they can find the matching Nix entry point
- AND they do not need to read Aspen archive notes first

#### Scenario: README routes readers to deeper docs by audience

- GIVEN a reader arrives at `README.md`
- WHEN they need consumer, lint-reference, or contributor details
- THEN the README links them to the matching deeper docs page
- AND it does not force every audience through one giant mixed document

### Requirement: Audience-Split Documentation Set

The repo MUST ship stable markdown docs for these audiences:

- consumers onboarding the first-class runner and config
- readers looking up lint defaults, domains, and suppression/reference details
- contributors maintaining lints, config keys, and published entry points

`docs/downstream-rollout.md` MAY stay as a deeper rollout playbook, but it MUST
NOT be the only consumer-facing doc page.

#### Scenario: Consumer finds config and rollout guidance without reading source

- GIVEN a consumer wants the zero-to-first-run path plus rollout guidance
- WHEN they follow the docs linked from `README.md`
- THEN they can find `[workspace.metadata.tigerstyle]`
- AND they can find `[tigerstyle]` / `dylint.toml` guidance
- AND they can find where `docs/downstream-rollout.md` fits as a deeper rollout reference

#### Scenario: Reader finds lint defaults and suppression guidance without reading source

- GIVEN a reader wants to know which lint family is on by default and how suppression works
- WHEN they follow the lint-reference docs
- THEN they can find the lint catalog grouped by domain
- AND they can find default levels and supported suppression/config paths
- AND they do not need to inspect `src/lib.rs` or `src/diagnostics.rs` first

### Requirement: Documentation Naming Contract

The docs MUST publish the current naming contract across the repo's public
surfaces.

That naming contract MUST distinguish at least these layers:

- repo/package name: `tigerstyle`
- binary name: `cargo-tigerstyle`
- cargo subcommand: `cargo tigerstyle check`
- lint namespace and groups: `tigerstyle::...`
- consumer config tables: `[tigerstyle]` and `[workspace.metadata.tigerstyle]`
- environment variables that remain part of the supported integration surface

This change's docs MUST treat a repo/package rename as out of scope and keep the
current names as canonical.

#### Scenario: Reader distinguishes project name from command and namespace

- GIVEN a reader asks whether tigerstyle should be renamed
- WHEN they read the naming section in the refreshed docs
- THEN they can tell which names are presentation text
- AND which names are current public contracts
- AND they can see that the docs refresh keeps the existing names in place

### Requirement: Documentation Maintenance Crosswalk

Contributor-facing docs MUST list the source-of-truth files and required doc
update touchpoints for shipped contract changes.

That crosswalk MUST cover at least these change types:

- adding or changing a lint
- adding or changing a config key
- adding or changing a published entry point

#### Scenario: Contributor adding a lint finds doc update touchpoints

- GIVEN a contributor adds or changes a lint
- WHEN they follow the contributor docs
- THEN they can find which source files define the lint contract
- AND which docs, templates, and spec files must be updated with it

#### Scenario: Contributor adding a config key or published entry point finds doc update touchpoints

- GIVEN a contributor adds a config key or changes a published entry point
- WHEN they follow the contributor docs
- THEN they can find the matching source-of-truth file
- AND the docs tell them which public reference pages and templates must change
