## Context

The repo has outgrown a single-document README. `README.md` currently carries four different jobs:

- landing page for new readers
- consumer quickstart
- full lint catalog
- maintainer/developer reference

At the same time, `docs/` only has `docs/downstream-rollout.md`, which is a deep rollout playbook, not a clean newcomer guide.

That mismatch now matters more because the public integration surface has grown:

- repo/package name: `tigerstyle`
- binary name: `cargo-tigerstyle`
- cargo subcommand: `cargo tigerstyle check`
- lint namespace: `tigerstyle::<lint>`
- consumer config tables: `[tigerstyle]` and `[workspace.metadata.tigerstyle]`
- environment variables: `TIGERSTYLE_*`

A reader who cannot see that map in one place will naturally ask whether the project itself should be renamed.

## Goals / Non-Goals

**Goals**

- make `README.md` a clean landing page
- split deep material into audience-specific docs
- document the naming contract clearly enough that new readers stop guessing
- give contributors a repeatable doc-update checklist when shipped contracts change

**Non-Goals**

- renaming the repo, package, binary, lint namespace, config keys, or env vars
- generating docs from source metadata in this change
- introducing a docs site generator or new build tooling

## Decisions

### 1. README becomes a landing page, not a full manual

**Choice:** Keep `README.md` short and front-load four answers:

1. what tigerstyle is
2. how a consumer runs it the first time
3. where deeper docs live
4. how the naming surface fits together

The README should still mention the lint domains and key migration notes, but it should link outward before it turns into a full manual.

**Rationale:** New readers need orientation before detail. The current README makes them scan a long catalog before they reach the first-class runner path.

**Alternative:** Keep all detail in `README.md` and only polish ordering. Rejected because the repo already has enough surface area that one long page will drift back into a mixed audience document.

### 2. Split docs by audience around the current public contract

**Choice:** Add stable markdown docs for three audiences:

- consumer quickstart/config reference
- lint catalog/reference
- contributor/maintainer workflow

Keep `docs/downstream-rollout.md` as the deep rollout playbook, linked from the consumer quickstart instead of acting as the only consumer doc.

**Rationale:** Consumers, maintainers, and contributors ask different questions. The current docs shape forces each audience to read through the others' material.

**Alternative:** Replace `docs/downstream-rollout.md` entirely. Rejected because the rollout playbook still carries useful Aspen-backed evidence and profile guidance.

### 3. Keep the project name `tigerstyle` and document the naming map instead of renaming now

**Choice:** This change will document the current naming contract and explicitly keep rename work out of scope.

The canonical naming table will live in `README.md`, with deeper docs linking back to it instead of duplicating conflicting variants.

That table or equivalent map will cover:

- repo/package: `tigerstyle`
- binary: `cargo-tigerstyle`
- command: `cargo tigerstyle check`
- lint attrs/groups: `tigerstyle::...`
- config tables: `[tigerstyle]`, `[workspace.metadata.tigerstyle]`
- relevant env vars: `TIGERSTYLE_*`

**Rationale:** Rename cost is high because these names are already part of consumer configs, source attrs, Nix outputs, tests, and saved Aspen guidance. Current confusion is a docs problem first.

**Alternative:** Rename the repo or package to match the binary. Rejected for this change because it would force a migration across every downstream contract at once.

### 4. Use contributor-facing crosswalks, not doc generation, to control drift

**Choice:** Add contributor guidance that names the source-of-truth files and required doc touchpoints for three common changes:

- adding or changing a lint
- adding or changing a config key
- adding or changing a published entry point

Expected source-of-truth anchors include `src/lib.rs`, `src/diagnostics.rs`, `src/config.rs`, `cargo-tigerstyle/`, `README.md`, `docs/`, `templates/dylint.toml`, and the relevant OpenSpec spec.

**Rationale:** Drift risk is real, but a handwritten crosswalk is much smaller than building a doc generator in the same change.

**Alternative:** Generate the lint catalog and config reference directly from Rust metadata. Rejected for now because it adds tooling scope before the docs information architecture is settled.

## Risks / Trade-offs

**Docs drift** -> contributor crosswalk plus deterministic audits over headings, key strings, and required links.

**README gets too thin** -> keep one-screen quickstart and naming map in README, then link deeper.

**Rename question returns later** -> acceptable. If external naming pain remains after docs refresh, open a dedicated migration change instead of smuggling rename work into a docs rewrite.
