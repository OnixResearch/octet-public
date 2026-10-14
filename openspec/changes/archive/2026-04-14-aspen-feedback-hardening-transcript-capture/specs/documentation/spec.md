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
- if the saved transcript starts with a command preamble, the repo MUST store a
  documented helper or exact capture command that emits that preamble
- consistency checks that verify the documented capture method, the saved
  helper, and the saved transcript agree on that preamble source
- check helpers that assert every summary claim the docs or verification notes
  make about the replay output, including zero-warning claims

#### Scenario: Reviewing transcript preamble provenance

- GIVEN a reviewer opens a saved reconstruction transcript whose first line is a
  `$ bash ...` command
- WHEN they inspect the saved helper scripts and consistency checks
- THEN they can see which documented helper emitted that preamble and verify the
  transcript was captured through that same helper path
