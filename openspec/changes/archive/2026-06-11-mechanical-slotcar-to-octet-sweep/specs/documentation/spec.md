## MODIFIED Requirements

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
