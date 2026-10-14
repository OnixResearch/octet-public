## Context

`ambient_clock` already treats time as hidden input. Environment variables, command-line arguments, and current working directory are the same class of ambient dependency and should be captured by shells before entering core logic.

## Decisions

### 1. Match direct `std::env` API use

**Choice:** The first implementation matches `std::env::{var,var_os,args,args_os,current_dir,set_current_dir,set_var,remove_var}` and resolved equivalents.

**Rationale:** These calls are visible, explainable, and map directly to an injection remediation.

### 2. Keep tests and shell boundaries quiet

**Choice:** Reuse test-context detection and allow explicit source-level suppressions with reasons for entry-point helpers.

**Rationale:** Tests and shells need ambient setup; the lint should focus production core paths.

### 3. Separate env from generic impure calls

**Choice:** Keep `ambient_env` as a standalone lint even if `impure_call_in_core` also covers env in core scopes.

**Rationale:** Environment access is common enough to deserve targeted guidance and rollout control.

## Risks / Trade-offs

- Some libraries intentionally read process context. Suppressions need local reasons.
- Facade crates may wrap env access; direct-call coverage is the first step.

## Validation

Use a pure path classifier with unit tests and UI fixtures proving both emitting and non-emitting shapes.
