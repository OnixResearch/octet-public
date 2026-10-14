## Context

Safety-lint broadening must avoid surprise churn. Each semantic expansion should be narrow, fixture-backed, and preserve existing suppressions. The design phases lint by lint so one noisy expansion does not block other high-value improvements.

## Decisions

### 1. Broaden result-discard detection first

**Choice:** Extend `ignored_result` and `drop_must_use` to recognize additional discard shapes such as statement-position `Result`, assignment to ignored bindings through blocks, and must-use producer calls hidden behind wrappers where rustc metadata is available.

**Rationale:** Silent error paths are high severity and common. These lints already have type information.

### 2. Add bounded recursion semantics conservatively

**Choice:** Extend `no_recursion` to methods and trait/default method contexts, then evaluate shallow mutual recursion as an allow-by-default advisory if direct expansion is too noisy.

**Rationale:** Direct method recursion is the same stack-risk class; full call-graph recursion may need a lower default level.

### 3. Make loop-bound recognition structural

**Choice:** Teach `unbounded_loop` to recognize explicit `break`, bounded counters, bounded iterator state, cancellation/shutdown channels, and bounded receiver loops through HIR/type facts.

**Rationale:** Source marker names are fragile and miss both real bounds and real liveness debt.

## Risks / Trade-offs

- Wider safety lints can create downstream noise. Mitigation: each phase ships with both emitting and non-emitting guardrails plus rollout notes.
- Some discard patterns overlap with rustc/clippy warnings. Octet should keep its diagnostic focused on Tiger Style remediation and stable rollout evidence.

## Validation

Run focused safety UI fixtures, async/Tokio fixtures for loop cases, pure helper tests, and dogfood summaries for changed lints before changing default levels.
