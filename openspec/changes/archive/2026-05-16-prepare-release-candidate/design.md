## Context

Recent work archived the Slotcar roadmap changes, restored the full Nix gate, and added release/operator smoke docs. The tree is ready for a release-candidate slice, but maintainers still need one deterministic receipt that proves candidate package outputs and install-smoke commands are present before tagging.

## Goals / Non-Goals

**Goals:**
- Add a small release candidate checklist/receipt document.
- Add CLI version output for install-smoke assertions.
- Add a Nix check that binds release docs, version output, and package outputs.
- Save OpenSpec evidence for the release-candidate smoke path.

**Non-Goals:**
- Publishing a GitHub release or pushing a tag.
- Changing version numbers beyond the current candidate metadata.
- Adding cryptographic signing or binary distribution.

## Decisions

### 1. Use checked-in release candidate docs plus a Nix check

**Choice:** Keep `docs/release-candidate.md` as the human checklist/receipt template and add `checks.x86_64-linux.release-candidate-smoke` as the deterministic gate.

**Rationale:** The check can prove the current flake builds the package surfaces and that installed binaries expose version output, while the checked-in doc gives maintainers and reviewers a stable place to inspect release notes and tag prerequisites.

**Alternative:** Generate all release notes dynamically. Rejected for this slice because there is no release publishing workflow yet and a static checklist is easier to review.

**Implementation:** Add a `runCommand` check in `flake.nix` that reads the doc from the flake source, asserts checklist lines, runs `${cargoSlotcarUnwrapped}/bin/cargo-slotcar --version` and `${cargoSlotcarUnwrapped}/bin/cargo-tigerstyle --version`, and references the lint library output.

### 2. Add root version handling to the CLI

**Choice:** Support `--version`, `-V`, and `version` at the root command surface.

**Rationale:** Release smoke should assert binary identity without relying on help text or Cargo metadata extraction in shell scripts.

**Alternative:** Only grep Cargo.toml in release checks. Rejected because it does not prove the shipped binary reports its version.

**Implementation:** Extend `CommandArgs` and `main` with a version command that prints program name plus `CARGO_PKG_VERSION`.

## Risks / Trade-offs

**Docs drift** → Mitigation: the Nix check greps required checklist headings and version lines.

**Alias confusion** → Mitigation: the smoke check verifies both `cargo-slotcar` and compatibility `cargo-tigerstyle` binaries report the same candidate version.

**Overclaiming release readiness** → Mitigation: this change explicitly stops at release-candidate verification; it does not tag, publish, sign, or deploy.
