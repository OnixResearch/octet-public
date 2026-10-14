## Contract

- [x] [serial] Specify the fact DTO + pure classifier + thin visitor boundary for hardened lint analysis. r[octet.lint_analysis_cores.functional_core]
- [x] [serial] Specify when source-snippet fallback is allowed and what evidence it needs. r[octet.lint_analysis_cores.snippet_fallback]

## Implementation

- [x] [serial] Refactor `public_unsafe_api` to classify public unsafe and extern APIs from HIR safety/ABI/visibility facts. r[octet.lint_analysis_cores.public_unsafe_api]
- [x] [serial] Refactor `multi_lock_ordering` to resolve lock fields structurally and require an order-specific comment shape. r[octet.lint_analysis_cores.multi_lock_ordering]
- [x] [serial] Refactor `unbounded_loop` to recognize structural bounds before source-marker fallback. r[octet.lint_analysis_cores.unbounded_loop]
- [x] [serial] Refactor `magic_number` policy-threshold classification around expression/binding facts. r[octet.lint_analysis_cores.magic_number]

## Verification

- [x] [parallel] Add pure positive and negative classifier tests for every refactored target lint. r[octet.lint_analysis_cores.functional_core]
- [x] [parallel] Add UI guardrail fixtures proving formatting, aliases, or unrelated comments no longer decide semantics. r[octet.lint_analysis_cores.snippet_fallback]
- [x] [serial] Run targeted library tests, UI rail, and Cairn validation/gates. r[octet.lint_analysis_cores.functional_core]
