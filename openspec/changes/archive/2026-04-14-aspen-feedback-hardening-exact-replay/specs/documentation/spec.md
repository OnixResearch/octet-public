# Documentation — Delta Spec

## MODIFIED Requirements

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
- any required sandbox, sibling-repo, or path-layout setup that the documented
  reconstruction path depends on
- archived text artifacts whose recorded commands reference the archived helper
  and evidence locations, not a deleted pre-archive change path
- if the documented reconstruction path is a saved helper command, a transcript
  whose top-level replay command matches that helper path exactly
- check helpers that assert every summary claim the docs or verification notes
  make about the replay output, including zero-warning claims

#### Scenario: Reviewing Aspen phase-1 reconstruction

- GIVEN a reviewer inspects the rollout-kit verification artifacts
- WHEN they open the reconstruction evidence
- THEN they can see the exact helper scripts, the temp-worktree setup, and the
  transcript proving the published Aspen phase-1 path ran from the new templates

#### Scenario: Reviewing archived reconstruction transcripts

- GIVEN the change has already been archived
- WHEN a reviewer opens the saved `.txt` transcripts and verification notes
- THEN the recorded commands point at the archived helper scripts and evidence
  files, not at a no-longer-active change directory

#### Scenario: Reviewing replay summary claims

- GIVEN the docs or verification notes claim `aspen-hlc: 0 starter findings`
  or another concrete replay summary
- WHEN a reviewer inspects the saved check helper
- THEN they can see an explicit assertion for that claim, not only a printed
  summary line
