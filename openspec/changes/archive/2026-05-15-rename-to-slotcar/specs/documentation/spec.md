## ADDED Requirements

### Requirement: Slotcar Public Documentation Identity

The system MUST present `slotcar` as the canonical repository, package, binary, cargo subcommand, Nix output, and documentation identity anywhere the text describes the product or tool that users install or invoke. Documentation MAY retain `Tiger Style` / `tigerstyle` only when the occurrence is explicitly about the coding philosophy, historical compatibility, lint namespace compatibility, or archived evidence.

The deterministic proof artifact `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt` MUST list every remaining non-archived `tigerstyle` occurrence after implementation as one of `philosophy`, `compatibility`, `namespace`, `archived`, or `needs-fix`, and the accepted artifact MUST contain zero `needs-fix` rows.

#### Scenario: Canonical install and usage docs say slotcar

- GIVEN a user reads current non-archived README, guides, templates, and generated examples
- WHEN the text tells them what to install, build, run, or configure as this repository's tool
- THEN the canonical name shown to the user is `slotcar`, `cargo-slotcar`, or `cargo slotcar`

#### Scenario: Philosophy references remain distinguishable

- GIVEN documentation still explains the Tiger Style coding philosophy
- WHEN a remaining `Tiger Style` or `tigerstyle` occurrence is not renamed
- THEN the occurrence is classified as philosophy, compatibility, namespace, or archived evidence rather than an unreviewed product-name leak
