# Octet release candidate checklist

This document is the pre-tag release candidate receipt for Octet. It is intentionally a local verification checklist; it does not tag, publish, sign, or deploy artifacts.

## Candidate identity

- Candidate version: `0.1.0`
- Canonical Cargo package: `cargo-octet`
- Compatibility Cargo binaries: `cargo-slotcar`, `cargo-tigerstyle`
- Lint library package output: `octet` with compatibility aliases `slotcar` and `tigerstyle`

## Open-source publication prerequisites

The maintenance owner is the Octet maintainer publishing the selected revision.
These checks prepare a repeatable source release; they do not authorize a tag,
repository visibility change, binary upload, or Git history rewrite. The
existing license grant is `AGPL-3.0-or-later`, not a new relicensing of imported
material. Keep `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`
with source distributions.

Before making the repository or a release public:

- Confirm the right to distribute original contributions, including employer
  permissions where applicable. Git authorship is not a copyright assignment.
- Keep the imported Adam Murray standards-prose attribution in
  [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). This public export
  excludes `docs/standards/source/` and carries `NOTICE`,
  `THIRD_PARTY_NOTICES.md`, and `LICENSES/` with the source distribution. The
  root AGPL notice alone is not treated as permission for imported prose.
- Fetch every exact Cargo and flake input from a clean, credential-free
  environment. On 2026-10-15 anonymous requests for the pinned Valence,
  Bounded Exec, Cairn, and Cairn artifact sources returned HTTP 404. This does
  not distinguish private, missing, or renamed sources. On 2026-11-11 the
  public cohort's pins did resolve anonymously, so the check is now a
  reproducible pass for this cohort rather than an open blocker; rerun it for
  every new pin.
- Review every branch/tag intended for publication and scan reachable history
  and the final tracked tree with a redacting secret scanner. Rotate/revoke
  any real credential before considering an approved history repair. Review
  personal paths, consumer data, and historical evidence separately; a scanner
  finding no credentials is not a privacy or publication clearance.
- Enable GitHub private vulnerability reporting and verify the channel in
  [SECURITY.md](../SECURITY.md); configure branch protection, review policy,
  and available secret-scanning/push-protection settings. Do not supply private
  deploy keys to pull-request-controlled builds.

The preparation scan used Gitleaks 8.30.1:

```bash
gitleaks git --redact=100 --no-banner --log-opts=--all .
gitleaks git --staged --redact=100 --no-banner .
```

At revision `4a540dff99aa2040e267e42bbafe09144f276572`, the history scan reported
1,316 commits and no findings; the staged scan also reported no findings.
`git rev-list --all --count` counted 1,362 reachable commits; scanner and Git
counts differ, so do not substitute one for the other. Rerun against the final
candidate. Unreachable objects, reflog-only history, external LFS/attachments,
and repository hosting settings were not cleared by these scans.

### Local Nix verification limit

On 2026-10-16, `nix flake check --no-write-lock-file -L` timed out waiting
for another Nix process to finish the pinned Bounded Exec fetch. A rerun
with an isolated `XDG_CACHE_HOME`, Git credential prompts disabled, and
empty Nix access tokens exited 1 while fetching
`https://github.com/OnixResearch/bounded-exec.git` at revision
`44eb74eb639194665ee820562d5b2ce2c3ff6a94`.

That isolated run also logged an `onix-artifact` fetch over SSH while
evaluating Cairn. The root artifact flake input uses HTTPS, but that does not
prove every transitive build source uses anonymous transport. Resolve this
path and exact source availability before claiming a credential-free build.
No full Nix package build or passing full flake gate is claimed.

### Binary and Nix distribution

Publish the exact matching corresponding source and build/install inputs,
including lockfiles, manifests, generated-input sources, build scripts, and
all required non-system dependency sources. Provide equivalent source access
and directions beside downloadable binaries as required by AGPL section 6.
An embedded lint-source subset is not the complete corresponding source for
the runner and its dependencies. Modified network-interactive versions also
need the source offer required by section 13.

Inventory license/notice obligations for the actual locked Rust and native/Nix
closure before uploading binaries. Octet packages install legal files under
`share/licenses/octet/`; Dylint driver and UI-driver outputs carry their own
license directories. These files are not an exhaustive closure audit. In
particular, review notice inclusion for the separately packaged Lean/leantar
tool outputs and any other redistributed tools before a binary release.

The supported distribution paths are a full source checkout and the pinned
Nix flake. Crates.io publication is not prepared: workspace path dependencies,
Git dependencies, and the runner's repository-wide embedded build inputs need
a separate packaging decision. Do not treat `cargo package --list` as proof
that a standalone crate can be built or published.

## Required pre-tag gates

Release operators should also review `docs/trust-boundary-evidence.md` for the
controlled-deployment posture, provenance fields, evidence tiers, and explicit
scope boundary behind release evidence claims.

Run these from the repository root before tagging:

```bash
nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link
nix flake check -L
cargo octet check --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

The public export excludes the imported prose source tree and carries the
required notice and license inventory.

## Install smoke commands

The release candidate smoke check verifies these installed binary surfaces from the current flake build:

```bash
nix run .#cargo-octet -- --version
nix run .#cargo-slotcar -- --version
nix run .#cargo-tigerstyle -- --version
nix build .#cargo-octet .#cargo-slotcar .#cargo-tigerstyle .#octet .#slotcar .#tigerstyle --no-link
```

For the supported public cohort, install from the pinned channel rather than a
local checkout. The command below is the copyable form; `main` is not a
supported pin.

```bash
nix run github:OnixResearch/octet-public/d5fb01535ad5b086942253320f3574ae4dd85883#cargo-octet -- --version
nix run github:OnixResearch/octet-public/d5fb01535ad5b086942253320f3574ae4dd85883#cargo-octet -- check
```

Expected version output:

```text
cargo-octet 0.1.0
```

## Release notes checklist

Before tagging, update release notes or the GitHub release body with:

- the candidate version (`0.1.0` for this candidate),
- the package outputs that shipped (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet`, `slotcar`, `tigerstyle`),
- the successful `release-candidate-smoke`, `nix flake check -L`, and local artifact verifier receipts,
- any known compatibility aliases or non-goals,
- the controlled-deployment and scope-boundary caveats for source/artifact locality and evidence claims.

## Acceptance signal

The candidate is ready to tag only after `checks.x86_64-linux.release-candidate-smoke` and `nix flake check -L` pass from a clean checkout and the OpenSpec evidence bundle records the exact commands and outputs.
