# Documentation Specification

## Purpose

Tiger Style documentation requirements for diagnostic wording, downstream
rollout guidance, namespace contracts, and consumer-facing rollout artifacts.
## Requirements
### Requirement: Standardized Diagnostic Structure

Every lint diagnostic MUST include:

1. A one-line warning message describing the violation
2. A help note in the exact form `Tiger Style (<principle>): <guidance>`;
   for lints with a concrete approved pattern, `<guidance>` MUST include an
   appended `Example: <approved pattern>` clause in the same help note
3. A note in the exact form `see Tiger Style guide: <section>`

#### Scenario: Developer sees lint for the first time

- GIVEN a developer runs tigerstyle and encounters `ambient_clock`
- WHEN they read the diagnostic
- THEN they see the principle name, the approved pattern, and where to
  read more without opening a separate document

#### Scenario: Diagnostic wording stays consistent across lints

- GIVEN two different tigerstyle lint diagnostics
- WHEN a developer compares their help and note lines
- THEN both diagnostics use the same `Tiger Style (...)` help prefix and
  the same `see Tiger Style guide: ...` note prefix

### Requirement: Canonical Tool-Scoped Tigerstyle Namespace

Every consumer-facing tigerstyle lint MUST be registered as a true tool lint in
canonical `tigerstyle::<lint_name>` form. Tool-scoped domain groups MUST be
registered against those true tool-lint IDs rather than ad-hoc plain-lint
aliases.

Standalone raw-rustc crates that use tool-scoped tigerstyle attrs MUST compile
on the supported nightly toolchain and opt into `#![feature(register_tool)]`
plus `#![register_tool(tigerstyle)]`.

#### Scenario: Tool-scoped per-lint attr resolves against a true tool lint

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN it uses a source-level attr like `#[allow(tigerstyle::no_unwrap)]`
- THEN rustc resolves that attr against the real `tigerstyle::no_unwrap` lint
  instead of a plain-lint compatibility shim

### Requirement: Tool-Lint Migration Precondition

Before converting all current consumer-facing lints, the project MUST validate
the planned migration mechanism on a single representative lint under the
repo's Dylint harness.

That precondition MUST prove both of these points together:

- `declare_tool_lint!(pub tigerstyle::...)` works for the representative lint
  under the project's Dylint loading path
- the intended bare compatibility mechanism — a one-member bare alias
  registered through `lint_store.register_group(...)` — still resolves the
  current bare config / lint-level path for that representative lint

#### Scenario: Representative spike still proves canonical and bare paths

- GIVEN representative lint `no_unwrap` has already proven the tool-lint spike
  under the repo's Dylint harness
- WHEN tigerstyle adds `no_todo` as another true tool lint with bare alias
  compatibility
- THEN the repo still follows the same validated migration pattern instead of
  inventing a second source-attr/config bridge

### Requirement: Lint Groups By Spec Domain

The project MUST register lint groups matching spec domains:

- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

Consumers MUST be able to use group-level `#[allow]` and `#[deny]`
attributes when compiling on the supported nightly toolchain and opting into
`#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]`.

#### Scenario: Suppressing a domain during phased rollout

- GIVEN a consumer in starter profile compiling on the supported nightly
  toolchain with `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN they add `#![allow(tigerstyle::safety)]` to a crate root
- THEN all safety-domain lints are suppressed for that crate

#### Scenario: Allowing one domain does not suppress another

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- AND the crate would otherwise trigger both a safety lint and a naming lint
- WHEN it adds `#![allow(tigerstyle::safety)]` to the crate root
- THEN the safety lint is suppressed
- AND the naming lint still emits a diagnostic

#### Scenario: Escalating one domain during cleanup

- GIVEN a consumer wants stricter naming cleanup and is compiling on the
  supported nightly toolchain with `#![feature(register_tool)]` and
  `#![register_tool(tigerstyle)]`
- WHEN they add `#![deny(tigerstyle::naming)]` to a crate root
- THEN naming-domain diagnostics in that crate become hard errors

### Requirement: Per-Lint Configuration Support

The project MUST continue to support disabling individual lints and overriding
lint severity through the consumer-owned `dylint.toml` even after migrating the
canonical lint namespace to `tigerstyle::<lint_name>`.

The consumer-owned `[tigerstyle]` section in `dylint.toml` MUST keep these
behaviors:

- `disabled_lints` continues to use bare lint names like `no_unwrap`
- `[tigerstyle.lint_levels]` continues to use bare lint names like `no_unwrap`
- bare config names resolve to the matching canonical tool lint without
  requiring consumers to rename config keys to `tigerstyle::...`
- `disabled_lints` still prevents disabled lints from registering, so it beats
  level overrides and source-level attrs

#### Scenario: Bare config key still upgrades the canonical tool lint

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code after the tool-lint migration
- THEN the canonical `tigerstyle::no_unwrap` lint emits an error instead of a
  warning

#### Scenario: Bare disabled key still beats source-level attrs

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- AND source code requests a source-level lint attribute for
  `tigerstyle::no_unwrap`
- WHEN the lint suite runs on code that would otherwise violate `no_unwrap`
- THEN `no_unwrap` still emits no diagnostic

#### Scenario: Disabled bare key keeps the lint out of groups and aliases

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite registers canonical domain groups and bare compatibility
  aliases
- THEN `no_unwrap` is omitted from those registrations
- AND no canonical group or bare compatibility path re-enables that lint

### Requirement: Rollout Kit Records New-Lint Aspen Classification

The downstream rollout kit MUST record Aspen evidence-backed classification for
newly added lint families before any starter-profile promotion.

The guidance MUST include:

- the per-lint evidence artifacts used for Aspen validation
- the classification of each pending lint family as `starter-candidate` or
  `inventory-first`
- the rule that inventory-first families stay disabled in starter-profile
  examples until a later promotion review lands
- any new lint-family names that consumers need to recognize during rollout

#### Scenario: Maintainer evaluates a newly added lint family

- GIVEN tigerstyle adds new warn-by-default lint families that are not yet in a
  starter rollout profile
- WHEN a maintainer follows the rollout kit
- THEN they can find the saved Aspen evidence, see the current classification
  for each family, and keep inventory-first families out of copied starter
  configs until later promotion work is complete

### Requirement: Rollout Kit Covers Low-Noise New-Lint Promotion

The downstream rollout kit MUST document how a lint family graduates from
`starter-candidate` classification into the copied starter profile.

The guidance MUST include:

- which low-noise new lint families are currently promoted into the starter
  profile
- which pending families remain inventory-first and therefore stay disabled in
  copied starter examples, README rollout text, and consumer template comments
- the documented starter package set that remains in scope for this promotion:
  `aspen-time`, `aspen-hlc`, and `aspen-core`
- saved Aspen evidence for the promoted starter profile, not only the earlier
  classification pass
- the first-class `cargo-tigerstyle` replay path used for that evidence
- deterministic evidence artifacts under the change `evidence/` directory:
  `replay-aspen-promoted-starter.sh`, `check-aspen-promoted-starter.py`,
  `aspen-promoted-starter-config.toml`,
  `aspen-promoted-starter-default.txt`,
  `aspen-promoted-starter-coordination.txt`, and
  `aspen-promoted-starter-counts.txt`
- separate saved proof for both the promoted starter scope and a focused
  `aspen-coordination` lib-only spotcheck that uses the same promoted lint set
- the exact promoted starter-profile configuration shape consumers should copy

#### Scenario: Maintainer copies the promoted starter profile

- GIVEN `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
  `negated_predicate` have already met the saved low-noise rule
- WHEN a maintainer copies the starter / boundary-first rollout example
- THEN those four families are enabled in that copied profile
- AND `unbounded_collection_growth` plus `too_many_parameters` remain disabled
  until a later promotion review lands

#### Scenario: Maintainer reviews promotion proof

- GIVEN a maintainer wants to confirm the promoted starter profile before
  copying it into a consumer workspace
- WHEN they follow the rollout kit
- THEN they can find the exact `cargo-tigerstyle` replay path
- AND separate saved transcripts for the promoted starter scope and the focused
  `aspen-coordination` lib-only spotcheck
- AND the checker/count artifact sits next to those transcripts under the
  change `evidence/` directory
- AND the proof still pins the starter package set to `aspen-time`,
  `aspen-hlc`, and `aspen-core`

### Requirement: Todo Escape Migration Guidance

The documentation set MUST identify `no_todo` as the supported
temporary-stub lint after the split in `README.md`,
`docs/downstream-rollout.md`, and the starter-template comments in
`templates/dylint.toml`.

That guidance MUST tell maintainers to migrate existing justified
`#[allow(tigerstyle::no_panic)]` and `#[allow(no_panic)]` `todo!()` escapes to
`#[allow(tigerstyle::no_todo)]` or the supported bare form `#[allow(no_todo)]`.

That guidance MUST also name the companion audit lint
`unjustified_no_todo_allow`, explain that `no_panic` now covers only
`panic!()`, `unimplemented!()`, and `unreachable!()`, and carry an explicit
breaking-change callout where the old `no_panic` stub escape hatch used to be
documented.

#### Scenario: Maintainer sees the `no_todo` migration path

- GIVEN a downstream workspace that already carries justified `todo!()` stubs
  under `allow(no_panic)` or `allow(tigerstyle::no_panic)`
- WHEN a maintainer follows the repo's rollout docs after this change
- THEN they see the required rename to `no_todo`
- AND they see the narrowed post-split meaning of `no_panic`
- AND they see an explicit breaking-change callout for the old stub escape
  hatch spelling

### Requirement: README Landing Page

The repo MUST keep `README.md` as a newcomer landing page instead of the only full manual.

That landing page MUST, near the top of the file:

- state what tigerstyle is
- show the first-class consumer run path with `cargo tigerstyle check` and the matching `nix run ...#cargo-tigerstyle -- check` form
- link to deeper docs by audience instead of embedding every rollout detail in the README itself
- identify the release-critical operator surfaces for `fingerprint`, `migrate`, effect-summary output, lint-cache behavior, and executable transcript examples
- point operators to one deterministic smoke transcript or docs-example gate that covers those release-critical surfaces together

#### Scenario: New user chooses an install/run path [r[readme-landing-page.scenario.new-user-chooses-an-install-run-path]]

- GIVEN a new user opens `README.md`
- WHEN they read the first setup section
- THEN they can choose between the installed `cargo tigerstyle check` path and the `nix run ...#cargo-tigerstyle -- check` path without reading archive specs

#### Scenario: Operator finds release-critical surfaces [r[readme-landing-page.scenario.operator-finds-release-critical-surfaces]]

- GIVEN an operator is preparing a release smoke run
- WHEN they read `README.md` and command help
- THEN they can find the `fingerprint`, `migrate`, effect-summary, lint-cache, and executable transcript surfaces from the release-facing docs/help path

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

- public site: `tigerstyle.dev`
- project name: `tigerstyle`
- GitHub repo reference: `github:OnixResearch/octet`
- crate/package name: `tigerstyle`
- binary name: `cargo-tigerstyle`
- cargo subcommand: `cargo tigerstyle check`
- lint namespace and groups: `tigerstyle::...`
- consumer config tables: `[tigerstyle]` and `[workspace.metadata.tigerstyle]`
- environment variables that remain part of the supported integration surface

This documentation contract MUST treat the current site, project, repo, crate,
binary, and command names as canonical unless a dedicated rename change says
otherwise.

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

### Requirement: Assertion Guide Discoverability

The public documentation set MUST expose the Tiger Style assertion guide and
copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Assertion review guide](docs/assertion-review.md)`
- `[Assertion checklist template](templates/assertion-checklist.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Assertion review guide](assertion-review.md)`
- `[Assertion checklist template](../templates/assertion-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

#### Scenario: Reader finds the assertion guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Assertion review guide](docs/assertion-review.md)` and
  `[Assertion checklist template](templates/assertion-checklist.md)`

#### Scenario: Contributor sees assertion doc touchpoints

- GIVEN a contributor changes public Tiger Style assertion guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Assertion review guide](assertion-review.md)`,
  `[Assertion checklist template](../templates/assertion-checklist.md)`, and
  `[Safety spec contract](../openspec/specs/safety/spec.md)`

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

### Requirement: Performance Guide Discoverability

The public documentation set MUST expose the Tiger Style performance guide and
copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Performance review guide](docs/performance-review.md)`
- `[Performance sketch template](templates/performance-sketch.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with exact
text and repo-valid destinations:

- `[Performance review guide](performance-review.md)`
- `[Performance sketch template](../templates/performance-sketch.md)`
- `[Performance spec contract](../openspec/specs/performance/spec.md)`

#### Scenario: Reader finds the performance guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Performance review guide](docs/performance-review.md)` and
  `[Performance sketch template](templates/performance-sketch.md)` without
  reading archived OpenSpec changes or private skill files

#### Scenario: Contributor sees performance doc touchpoints

- GIVEN a contributor changes public Tiger Style guidance for performance review
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Performance review guide](performance-review.md)`,
  `[Performance sketch template](../templates/performance-sketch.md)`, and
  `[Performance spec contract](../openspec/specs/performance/spec.md)` in the
  maintained documentation crosswalk

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

### Requirement: Bounds And Pacing Guide Discoverability

The public documentation set MUST expose the Tiger Style bounds-and-pacing
guide and copyable template from the repo's documented entry points.

`README.md` MUST contain these literal Markdown links with exact text and
repo-valid destinations:

- `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)`
- `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`

`docs/contributor-guide.md` MUST contain these literal Markdown links with
exact text and repo-valid destinations:

- `[Bounds and pacing review guide](bounds-and-pacing-review.md)`
- `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

#### Scenario: Reader finds the bounds guide from README

- GIVEN a new reader opens `README.md`
- WHEN they look for Tiger Style material beyond lint catalogs and rollout docs
- THEN they can find the exact Markdown links
  `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)` and
  `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`

#### Scenario: Contributor sees bounds doc touchpoints

- GIVEN a contributor changes public Tiger Style boundedness guidance
- WHEN they follow `docs/contributor-guide.md`
- THEN they can find the exact Markdown links
  `[Bounds and pacing review guide](bounds-and-pacing-review.md)`,
  `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`,
  and `[Safety spec contract](../openspec/specs/safety/spec.md)`

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

### Requirement: Fixture Backed Standards Examples [r[fixture-backed-standards-examples]]

The system MUST require standards code examples that claim checker behavior to link to fixture-backed evidence or an explicitly marked prose-only exemption.

#### Scenario: Example links to fixture [r[fixture-backed-standards-examples-scenario-example-links-to-fixture]]

- GIVEN a standards page shows a bad or good Rust example for a checker behavior
- WHEN example verification runs
- THEN the example maps to a fixture path, expected output path, and content hashes

#### Scenario: Prose-only example is explicit [r[fixture-backed-standards-examples-scenario-prose-only-example-is-explicit]]

- GIVEN an example is illustrative and not executable
- WHEN documentation verification runs
- THEN the example is marked prose-only with a rationale rather than silently counted as verified

### Requirement: Live Example Verification Command [r[live-example-verification-command]]

The system MUST provide or document a local-first command that validates example manifests against checked-in fixtures and expected outputs.

#### Scenario: Changed fixture invalidates manifest [r[live-example-verification-command-scenario-changed-fixture-invalidates-manifest]]

- GIVEN a fixture body changes without manifest hash update
- WHEN live example verification runs
- THEN the command fails and names the stale example entry

### Requirement: Octet Public Documentation Identity Baseline

The system MUST present `Octet` / `octet` as the canonical repository, package, binary, cargo subcommand, Nix output, and documentation identity anywhere the text describes the product or tool that users install or invoke. Documentation MAY retain `Tiger Style`, `tigerstyle`, or `slotcar` only when the occurrence is explicitly about the coding philosophy, historical compatibility, lint namespace compatibility, a transition alias, or archived evidence.

The deterministic proof artifact `openspec/changes/archive/2026-05-18-rename-slotcar-to-octet/evidence/name-classification.txt` MUST list every remaining non-archived `slotcar`, `Slotcar`, `tigerstyle`, or `Tiger Style` occurrence after implementation as one of `philosophy`, `compatibility`, `namespace`, `archived`, or `needs-fix`, and the accepted artifact MUST contain zero `needs-fix` rows.

#### Scenario: Canonical install and usage docs say octet

- GIVEN a user reads current non-archived README, guides, templates, and generated examples
- WHEN the text tells them what to install, build, run, or configure as this repository's tool
- THEN the canonical name shown to the user is `octet`, `cargo-octet`, or `cargo octet`

#### Scenario: Philosophy references remain distinguishable

- GIVEN documentation still explains the Tiger Style coding philosophy
- WHEN a remaining `Tiger Style` or `tigerstyle` occurrence is not renamed
- THEN the occurrence is classified as philosophy, compatibility, namespace, or archived evidence rather than an unreviewed product-name leak

### Requirement: Executable Documentation Transcripts [r[executable-documentation-transcripts]]

The system MUST support executable documentation transcripts for Octet examples. A transcript MUST be a Markdown document containing Octet command blocks, fixture setup references, and expected-output assertions. Running a transcript MUST produce deterministic output artifacts suitable for tests and OpenSpec evidence.

#### Scenario: Transcript executes command blocks [r[executable-documentation-transcripts.scenario.transcript-executes-command-blocks]]

- GIVEN a docs example contains an `octet-check` command block and fixture reference
- WHEN `cargo octet transcript <path>` runs
- THEN the command executes in an isolated fixture workspace and records exit status plus bounded output

#### Scenario: Expected output assertion fails closed [r[executable-documentation-transcripts.scenario.expected-output-assertion-fails-closed]]

- GIVEN a transcript expected-output block names a lint that no longer appears
- WHEN the transcript runner evaluates the assertion
- THEN the command exits non-zero and reports the failed assertion location

#### Scenario: Transcript artifact is content addressed [r[executable-documentation-transcripts.scenario.transcript-artifact-is-content-addressed]]

- GIVEN the same transcript, fixture, command, and tool revision are run twice
- WHEN the transcript runner writes its artifact header
- THEN the transcript hash fields are identical across runs

#### Scenario: Release smoke transcript covers operator surfaces [r[executable-documentation-transcripts.scenario.release-smoke-transcript-covers-operator-surfaces]]

- GIVEN an operator wants one deterministic release-readiness smoke artifact
- WHEN they run the documented smoke transcript or docs-example gate
- THEN the saved transcript output names `fingerprint`, `migrate`, effect-summary, lint-cache, and executable transcript behavior in one bounded artifact

### Requirement: Release Candidate Readiness [r[release-candidate-readiness]]

The repository MUST provide a deterministic release candidate readiness path that a maintainer can run before tagging a release.

The readiness path MUST identify the candidate version, the package/app outputs that are expected to ship, the release notes or changelog checklist, and at least one install-smoke command for the shipped Cargo subcommand surface.

#### Scenario: Maintainer verifies package outputs before tagging [r[release-candidate-readiness.scenario.maintainer-verifies-package-outputs-before-tagging]]

- GIVEN a maintainer is preparing an Octet release candidate
- WHEN they run the release candidate smoke check
- THEN the check builds or references the canonical `cargo-octet` plus compatibility `cargo-slotcar`, `cargo-tigerstyle`, and `octet`/`slotcar`/`tigerstyle` package outputs from the current flake
- AND the saved receipt names the candidate version and release notes checklist

#### Scenario: Installed binary reports candidate version [r[release-candidate-readiness.scenario.installed-binary-reports-candidate-version]]

- GIVEN the candidate Cargo subcommand binary has been built from the release tree
- WHEN a maintainer runs the documented version smoke command
- THEN the command exits successfully and reports the same version as the candidate package metadata

#### Scenario: Release candidate receipt is saved as review evidence [r[release-candidate-readiness.scenario.release-candidate-receipt-is-saved-as-review-evidence]]

- GIVEN the release candidate smoke check has passed
- WHEN a reviewer inspects the OpenSpec evidence bundle
- THEN it contains the exact commands and outputs used to prove package outputs, version output, release notes coverage, and install smoke readiness

### Requirement: Structured Session Todo State [r[structured-session-todo-state]]

The project MUST document and expose structured session todo states for pending edits, failing checks, unresolved review findings, stale specs, and completed work. The state model MUST make unresolved work distinguishable from completed or intentionally deferred work.

#### Scenario: Session todo state is structured [r[structured-session-todo-state.scenario.todo-state-structured]]

- GIVEN an agent session has pending edits, failing checks, unresolved review findings, and stale OpenSpec tasks
- WHEN session todo state is recorded or displayed
- THEN each item has a stable kind, status, source reference, and next action
- AND the output is machine-readable enough for follow-up agents to resume without parsing prose summaries

#### Scenario: Resolved and unresolved work remain distinguishable [r[structured-session-todo-state.scenario.resolved-unresolved-distinguishable]]

- GIVEN a task was completed, another was deferred, and another remains blocked
- WHEN the session state is inspected later
- THEN completed, deferred, and blocked items are represented as separate states
- AND blocked items preserve the blocker reason and required next evidence

### Requirement: Replay Capsule Operator Guidance [r[replay-capsule-operator-guidance]]

The system MUST document how operators capture, inspect, and replay Octet replay capsules for lint and fingerprint evidence. The guidance MUST name exact commands, required inputs, redaction guarantees, and evidence paths suitable for OpenSpec review.

#### Scenario: Operator guide names capture and replay commands [r[replay-capsule-operator-guidance.scenario.commands-named]]

- GIVEN an operator needs to preserve a release, review, or bug-reproduction Octet run
- WHEN they read the replay-capsule documentation
- THEN the guide names the capture, inspect, and replay commands
- AND it states which capsule fields are required for `cargo octet check` and `cargo octet fingerprint` evidence

#### Scenario: Operator guide states replay limits [r[replay-capsule-operator-guidance.scenario.limits-stated]]

- GIVEN a replay capsule reports `match`
- WHEN an operator reads the guidance
- THEN the guidance states that the capsule proves reproducible evidence only
- AND it does not claim semantic correctness, safety, or verification solely from matching hashes

### Requirement: Review Identity Operator Guidance [r[review-identity-operator-guidance]]

The system MUST document how operators create, inspect, update, and roll back Octet review identity registry entries. The guidance MUST explain the distinction between mutable labels and immutable target hashes.

#### Scenario: Guidance explains name and hash roles [r[review-identity-operator-guidance.scenario.name-hash-roles]]

- GIVEN an operator reads the review identity guide
- WHEN the guide introduces labels and target hashes
- THEN it states that labels are ergonomic mutable pointers
- AND immutable hashes are the durable evidence identity used for validation and replay

#### Scenario: Guidance documents rollback [r[review-identity-operator-guidance.scenario.rollback]]

- GIVEN an operator needs to revert a review label to a previous evidence target
- WHEN they read the guide
- THEN the guide names the inspect and rollback workflow
- AND it states that rollback changes the pointer only, not the immutable target records

### Requirement: Octet Public Documentation Identity

The system MUST present `Octet` / `octet` as the canonical repository, package, binary, cargo subcommand, Nix output, configuration, receipt, and documentation identity anywhere current non-archived text describes the product, toolchain, or tool that users install, invoke, configure, or integrate.

A Steel-backed identity sweep MUST be the mechanical path for replacing active `slotcar` / `Slotcar` product identity with `octet` / `Octet`. The sweep report MUST be deterministic and MUST list each matched occurrence as `rewritten`, `allowed-compatibility`, `allowed-archive`, `allowed-philosophy`, or `blocked`. The accepted final report MUST contain zero `blocked` rows and zero unapproved active `slotcar` / `Slotcar` occurrences.

Archived evidence under `openspec/changes/archive/**`, `cairn/archive/**`, and comparable historical bundles MUST stay historically faithful: the sweep MAY scan and classify those files, but it MUST NOT rewrite them unless a current maintained spec or guide points at a dead active path.

#### Scenario: Steel dry-run inventories active slotcar references

- GIVEN the repository contains active files with `slotcar` or `Slotcar` strings
- WHEN the Steel sweep runs in dry-run mode
- THEN the report lists the repo-relative path, line, matched text, planned replacement or allow category, and reason for every active current-tree occurrence
- AND no files are modified

#### Scenario: Steel apply rewrites active documentation identity

- GIVEN the dry-run report has no unexpected blocked rows
- WHEN the Steel sweep runs in apply mode
- THEN current docs, templates, main OpenSpec specs, and generated examples use `Octet` / `octet` for product identity
- AND the apply report lists the exact file count and occurrence count rewritten

#### Scenario: Remaining slotcar terms are explicitly allowed

- GIVEN the apply mode has completed
- WHEN the Steel sweep runs in verify mode
- THEN any remaining active `slotcar` / `Slotcar` occurrence appears only as `allowed-compatibility`, `allowed-archive`, or another named allow category with a reason
- AND the report contains zero `blocked` rows

#### Scenario: Archived evidence remains historical

- GIVEN archived evidence contains historical `slotcar` command lines, paths, or transcripts
- WHEN the Steel sweep runs in apply mode
- THEN those archived files are not rewritten
- AND the final report classifies them as `allowed-archive` rather than active product identity leaks
