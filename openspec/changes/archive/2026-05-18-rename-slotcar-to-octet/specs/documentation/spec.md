## ADDED Requirements

### Requirement: Octet Public Documentation Identity

The system MUST present `Octet` / `octet` as the canonical repository, package, binary, cargo subcommand, Nix output, configuration, receipt, and documentation identity anywhere current text describes the product, toolchain, or tool that users install, invoke, configure, or integrate.

Documentation MUST explain the Octet metaphor as stable-shell composition: Rust-compatible code is inert by default across APIs, effects, capabilities, proofs, and review surfaces, and interaction occurs only through complete, explicit interfaces. Documentation MAY retain `Tiger Style`, `tigerstyle`, or `slotcar` only when the occurrence is explicitly about source philosophy, historical compatibility, lint namespace compatibility, archived evidence, or a transition alias.

The deterministic proof artifact `openspec/changes/archive/2026-05-18-rename-slotcar-to-octet/evidence/name-classification.txt` MUST list every remaining non-archived `slotcar`, `Slotcar`, `tigerstyle`, or `Tiger Style` occurrence after implementation as one of `philosophy`, `compatibility`, `namespace`, `archived`, or `needs-fix`, and the accepted artifact MUST contain zero `needs-fix` rows.

#### Scenario: Canonical install and usage docs say Octet

- GIVEN a user reads current non-archived README, guides, templates, and generated examples
- WHEN the text tells them what to install, build, run, configure, or integrate as this repository's toolchain
- THEN the canonical name shown to the user is `Octet`, `octet`, `cargo-octet`, or `cargo octet`

#### Scenario: Chemistry metaphor explains stable-shell interfaces

- GIVEN a user reads the project overview or consumer guide
- WHEN the text explains why the project is named Octet
- THEN it describes stable shells, inert-by-default behavior, or complete explicit interfaces for effects, capabilities, APIs, proofs, or review surfaces rather than a cosmetic rename only

#### Scenario: Legacy names remain distinguishable

- GIVEN current docs still contain `Tiger Style`, `tigerstyle`, `slotcar`, or `Slotcar`
- WHEN deterministic name classification runs
- THEN every remaining occurrence is classified as philosophy, compatibility, namespace, or archived evidence rather than an unreviewed product-name leak
