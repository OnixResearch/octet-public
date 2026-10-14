## Why

The safety family carries Octet's highest-risk static checks, but several lints still cover only narrow syntactic shapes. `ignored_result` focuses on `let _ = Result`, `drop_must_use` focuses on explicit `drop(...)`, `no_recursion` catches direct self-calls, and `unbounded_loop` relies partly on visible source markers.

These checks should recognize more real failure shapes without becoming noisy. The next improvement is semantic depth: broader discard detection, method and mutual recursion signals, and structural loop bounds.

## What Changes

- Specify deeper safety-lint semantics for ignored results, must-use drops, recursion, and unbounded loops.
- Phase implementation by lint, with each phase requiring positive and negative fixtures before broadening diagnostics.
- Preserve existing suppressions and test-context behavior.

## Impact

- **Files**: `src/safety/ignored_result.rs`, `drop_must_use.rs`, `no_recursion.rs`, `unbounded_loop.rs`, `test_context.rs`, UI fixtures, and lint metadata/docs where summaries change.
- **Testing**: pure helper tests where practical, UI positive/negative fixtures, async/Tokio fixture support for loop cases, and targeted safety UI reruns.
