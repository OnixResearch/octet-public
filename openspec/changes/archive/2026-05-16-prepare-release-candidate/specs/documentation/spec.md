## ADDED Requirements

### Requirement: Release Candidate Readiness [r[release-candidate-readiness]]

The repository MUST provide a deterministic release candidate readiness path that a maintainer can run before tagging a release.

The readiness path MUST identify the candidate version, the package/app outputs that are expected to ship, the release notes or changelog checklist, and at least one install-smoke command for the shipped Cargo subcommand surface.

#### Scenario: Maintainer verifies package outputs before tagging [r[release-candidate-readiness.scenario.maintainer-verifies-package-outputs-before-tagging]]

- GIVEN a maintainer is preparing a Slotcar release candidate
- WHEN they run the release candidate smoke check
- THEN the check builds or references the `cargo-slotcar`, `cargo-tigerstyle`, and `tigerstyle`/`slotcar` package outputs from the current flake
- AND the saved receipt names the candidate version and release notes checklist

#### Scenario: Installed binary reports candidate version [r[release-candidate-readiness.scenario.installed-binary-reports-candidate-version]]

- GIVEN the candidate Cargo subcommand binary has been built from the release tree
- WHEN a maintainer runs the documented version smoke command
- THEN the command exits successfully and reports the same version as the candidate package metadata

#### Scenario: Release candidate receipt is saved as review evidence [r[release-candidate-readiness.scenario.release-candidate-receipt-is-saved-as-review-evidence]]

- GIVEN the release candidate smoke check has passed
- WHEN a reviewer inspects the OpenSpec evidence bundle
- THEN it contains the exact commands and outputs used to prove package outputs, version output, release notes coverage, and install smoke readiness
