# Tiger Style Reference → Skill Crosswalk

Reviewed: 2026-04-12
Reference: `/path/to/.pi/agent/skills/tigerstyle/references/tigerstyle.md`
Skill: `/path/to/.pi/agent/skills/tigerstyle/SKILL.md`

## Purpose

This crosswalk maps each major normative rule set from the reference to the
compressed skill. It also records what remains reference-only on purpose.

## Safety

| Reference section | Skill coverage |
|---|---|
| Control and limits: simple control flow, fixed limits, function length, centralize control flow, functional core / imperative shell | `## Core Principles > Safety`; `## Compiler-Enforced Core/Shell`; `## Applying Tiger Style` |
| Memory and types: explicit types, static allocation, minimize variable scope | `## Core Principles > Safety`; `## Developer Experience` |
| Error handling: assertion density, positive/negative space, split assertions, implication assertions, compile-time assertions, assert args/returns, validate invariants, pair assertions, fail fast, handle all errors, test valid/invalid, warnings as errors, avoid implicit defaults | `## Core Principles > Safety`; `## Applying Tiger Style`; `When reviewing code` |
| Control flow safety: buffer external events, decompose conditions, state invariants positively | `## Core Principles > Safety`; `## Applying Tiger Style` |
| Concurrency safety: lock order, deterministic peer ordering, typed protocol state, quarantine unsafe, loud escape hatches | `## Core Principles > Safety`; `## Applying Tiger Style`; `When reviewing code` |

## Performance

| Reference section | Skill coverage |
|---|---|
| Design for performance: early design, napkin math, bandwidth and latency, control plane vs data plane, batching | `## Core Principles > Performance`; `## Applying Tiger Style` |
| Efficient resource use: resource priority, cost × frequency | `## Core Principles > Performance`; `When reviewing code` |
| CPU efficiency: large work chunks, primitive-argument hot loops | `## Core Principles > Performance` |
| Predictability: predictable paths, reduced compiler dependence | `## Core Principles > Performance`; `When reviewing code` |

## Developer experience

| Reference section | Skill coverage |
|---|---|
| Name things: naming style, abbreviations, acronym style, units, behavioral naming, no name overloading, option structs, explain why, comment style | `## Core Principles > Developer Experience`; `## Applying Tiger Style`; `When reviewing code` |
| Organize things: public API first, helper prefixes, callbacks last, simplify signatures, construct in place, minimize variable scope | `## Core Principles > Developer Experience`; `## Applying Tiger Style` |
| Ensure consistency: no duplicates, pass large objects by reference, minimize dimensionality, clean buffer allocation | `## Core Principles > Developer Experience`; `## Applying Tiger Style`; `When reviewing code` |
| Avoid off-by-one errors: indexes/counts/sizes, intentional division | `## Core Principles > Developer Experience` |
| Code consistency and tooling: indentation, line lengths, clear code blocks, bracing, long-form flags, Rust tooling, minimal deps, standardized tooling | `## Core Principles > Developer Experience` |
| Commit and test documentation: standalone commit messages, document test methodology | `## Core Principles > Developer Experience`; `## Applying Tiger Style` |
| Zero technical debt addendum: do it right first time, proactive, build momentum | `## Core Principles > Developer Experience`; `When reviewing code` |

## Purity / Verus

| Reference section | Skill coverage |
|---|---|
| Compiler-enforced core/shell via workspace crates | `## Compiler-Enforced Core/Shell via Workspace Crates` |
| Read → Transform → Write workflow | same section |
| Verus: when to use, inline ensures, split two-file pattern, spec/exec/proof/external_body, state-machine invariant proof pattern, commands, best practices | `## Formal Verification with Verus` |
| Functional core enables testing, property testing, verification | `## Core Principles > Safety`; `## Formal Verification with Verus`; `## Applying Tiger Style` |

## Intentionally reference-only material

These stay primarily in the reference so the skill remains skimmable:

- long rationale paragraphs for each rule
- extended multi-paragraph examples and anti-examples
- full worked napkin-math walkthrough details
- long Verus sample code beyond the compact patterns in the skill
- colophon, license, and source provenance

## Validation notes

- Checked that every gap identified in the 2026-04-12 review now has an explicit home in the skill.
- Checked that the skill points back to `references/tigerstyle.md` for extended examples and rationale.
- Checked that no major normative section from the reference is represented only by the reference.
