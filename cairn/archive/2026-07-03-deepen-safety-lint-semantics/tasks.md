## Contract

- [x] [serial] Specify broadened discard semantics for `ignored_result` and `drop_must_use`. r[octet.safety_lint_semantics.discards]
- [x] [serial] Specify method/direct recursion coverage and advisory boundaries for mutual recursion. r[octet.safety_lint_semantics.recursion]
- [x] [serial] Specify structural loop-bound recognition for `unbounded_loop`. r[octet.safety_lint_semantics.loop_bounds]

## Implementation

- [x] [serial] Extend `ignored_result` and `drop_must_use` to cover additional typed discard forms while preserving shutdown/drop-context exceptions. r[octet.safety_lint_semantics.discards]
- [x] [serial] Extend `no_recursion` to method self-recursion and document or implement mutual-recursion advisory behavior. r[octet.safety_lint_semantics.recursion]
- [x] [serial] Replace source-marker loop checks with structural bound/cancellation recognition where type facts are available. r[octet.safety_lint_semantics.loop_bounds]

## Verification

- [x] [parallel] Add positive and negative UI fixtures for every new discard shape. r[octet.safety_lint_semantics.discards]
- [x] [parallel] Add method recursion and non-recursive trait/default-method guardrail fixtures. r[octet.safety_lint_semantics.recursion]
- [x] [parallel] Add bounded-loop, shutdown-channel, and runaway-loop fixtures, including async/Tokio cases. r[octet.safety_lint_semantics.loop_bounds]
- [x] [serial] Run targeted safety UI rail, `cargo test --lib`, and Cairn validation/gates. r[octet.safety_lint_semantics.loop_bounds]
