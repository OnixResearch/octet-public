# Expand Tiger Style Skill Coverage

## Why

`/path/to/.pi/agent/skills/tigerstyle/SKILL.md` captures the big Tiger Style themes but drops many actionable rules from `references/tigerstyle.md`. Missing guidance spans safety, performance, developer experience, and Verus workflow. Result: agents get uneven instruction, apply Tiger Style partially, and miss patterns the full reference treats as normative.

Current gaps include conditional assertion patterns, warnings-as-errors, implicit-default avoidance, bandwidth-vs-latency analysis, frequency-weighted optimization, behavioral naming, options structs for ambiguous parameters, formatting rules, commit/test documentation rules, and the fuller Verus two-file workflow.

## What Changes

- Define spec coverage requirements for the Tiger Style skill so every major normative rule in the reference is represented in the skill.
- Add new spec domains for `performance` and `documentation` to cover missing guidance areas.
- Expand existing domain specs so the skill includes the missing safety, naming, structure, purity, and concurrency details.
- Require a repeatable coverage check or crosswalk so future edits do not silently drop reference guidance.

## Capabilities

### New Capabilities

- `tigerstyle-skill-completeness`: Skill covers all major normative Tiger Style guidance from the reference
- `tigerstyle-performance-guidance`: Skill includes full performance guidance, not only high-level slogans
- `tigerstyle-verus-workflow`: Skill documents both inline and two-file Verus patterns
- `tigerstyle-doc-crosswalk`: Skill changes are checked against the reference for omissions

## Impact

- **Files**: `openspec/specs/`, `openspec/config.yaml`, Tiger Style skill files under `~/.pi/agent/skills/tigerstyle/`
- **APIs**: No Rust API changes
- **Dependencies**: None required
- **Testing**: Reference-to-skill crosswalk review, section-by-section diff, example spot checks
