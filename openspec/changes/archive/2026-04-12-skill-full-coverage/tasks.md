# Tasks: Expand Tiger Style Skill Coverage

## Phase 1: Spec taxonomy

- [x] Add `performance` and `documentation` domains to `openspec/config.yaml`
- [x] Add main spec files for new domains under `openspec/specs/`
- [x] Sync existing main specs so safety, naming, structure, purity, and concurrency include the new skill-coverage requirements

## Phase 2: Skill coverage audit

- [x] Build a section-by-section reference → skill crosswalk covering all major normative rules in `references/tigerstyle.md`
- [x] Mark each rule group as safety, performance, naming, structure, purity, concurrency, or documentation
- [x] Record intentionally reference-only material as examples, rationale, or extended background

## Phase 3: Skill content expansion

- [x] Expand safety guidance in `~/.pi/agent/skills/tigerstyle/SKILL.md` to include missing assertion patterns, argument/return checks, invariant checks, fail-fast rules, exhaustive invalid-path testing, warnings-as-errors, implicit-default avoidance, positive predicates, bounded event handling, and compound-condition decomposition
- [x] Expand performance guidance in the skill to include bandwidth vs latency, cost × frequency, predictability, compiler-independence, and an explicit pointer to worked napkin math guidance
- [x] Expand naming guidance in the skill to include behavioral naming, name-overload avoidance, options structs for ambiguous parameters, and proper comment style
- [x] Expand structure guidance in the skill to include simplified signatures, in-place construction, passing large objects by reference, minimizing dimensionality, clean buffer allocation, explicit division intent, formatting rules, standalone commit messages, and test methodology guidance
- [x] Expand Verus guidance in the skill to include the split `verus/` pattern, `spec fn` / `exec fn` / `proof fn`, `#[verifier(external_body)]`, invariant proof patterns, verification commands, and best practices
- [x] Expand concurrency guidance in the skill to include quarantining `unsafe` behind safe APIs
- [x] Add explicit pointers from the skill to `references/tigerstyle.md` for extended examples, rationale, and long-form walkthroughs

## Phase 4: Validation

- [x] Review updated skill against the crosswalk and confirm no major normative gaps remain
- [x] Spot-check that the expanded skill remains skimmable and does not duplicate large examples unnecessarily
- [x] Record validation notes in the crosswalk artifact
