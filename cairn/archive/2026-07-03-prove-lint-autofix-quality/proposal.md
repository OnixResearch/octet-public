## Why

Many lints now emit structured suggestions, but only a small subset has checked `.fixed` artifacts and idempotence proof. Suggestions are user-visible behavior: a wrong `MachineApplicable` fix is worse than a warning because it can rewrite code incorrectly.

Octet needs a stronger autofix quality contract that separates machine-applicable fixes from advisory replacements and proves every accepted machine fix is stable.

## What Changes

- Add an autofix quality contract covering applicability tiers, `.fixed` fixture coverage, idempotence, and warning-free fixed outputs.
- Require every `MachineApplicable` suggestion to have a checked `.fixed` artifact unless explicitly classified as out of scope with a rationale.
- Extend rollout evidence for MaybeIncorrect and HasPlaceholders suggestions so users can trust the tier labels.

## Impact

- **Files**: `src/diagnostics.rs`, lint modules emitting suggestions, `ui_tests/*.fixed`, `tests/ui.rs`, `tests/fixtures/*`, and docs describing autofix behavior.
- **Testing**: ignored UI rail, `autofix_machine_applicable`, idempotence reruns, and negative tests proving placeholder fixes are not treated as machine-applicable.
