# Documentation Skill Coverage — Delta Spec

## ADDED Requirements

### Requirement: Tiger Style Skill Coverage Crosswalk

The project MUST maintain an explicit crosswalk from the Tiger Style
reference document to the agent skill.

The crosswalk MUST identify, for each major normative rule or grouped
rule set in the reference, where that guidance appears in the skill.

The crosswalk MUST call out any intentionally reference-only material
as examples, rationale, or extended background.

#### Scenario: Reviewing a future skill edit

- GIVEN the skill has been edited
- WHEN a maintainer verifies coverage
- THEN the crosswalk shows whether any reference guidance was dropped

### Requirement: Skill Completeness Without Verbatim Duplication

The Tiger Style skill MUST remain skimmable while still covering all
major normative guidance from the reference.

The skill MAY group related rules into concise checklists.

The skill MUST provide explicit pointers to the reference for extended
examples, rationale, and long-form walkthroughs.

The skill MUST NOT rely on the reference to carry normative rules that
are absent from the skill.

#### Scenario: Agent uses skill only

- GIVEN an agent reads the skill but not the full reference
- WHEN it applies Tiger Style during coding or review
- THEN it still sees all major normative rules needed for correct application
