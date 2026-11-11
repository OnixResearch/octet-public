# Boolean API migration inventory

This is the bounded source inventory for `raw_bool_parameter` and its
suppression guard. It records compiled findings and the chosen migrations; it
does not claim that every other Octet lint is clean, that any external consumer
is migrated, or that a clean two-lint run is a release gate.

## Octet workspace

The reviewed work on
`origin/review/harden-boolean-api-modeling-unverified-20261002` identified
private and public boolean-parameter sites in Octet's own sources. This change
carried those migrations onto the current base before the first compiled
inventory:

| Site | Initial shape | Migration |
| --- | --- | --- |
| `crates/octet-standards/src/project_architecture_gate.rs` — `classify_status` | Two private positional flags | Derive integration and policy status from the post-waiver finding slice at the classification boundary. |
| `tests/support/consumer_fixture.rs` — `run_internal` | Two private positional flags | Use `LockfileMutation` for the mutually exclusive check/build mutation modes. |
| `src/safety/no_todo.rs` — `NoTodo::new` | Two public direct-`bool` parameters | Pass a named `NoTodoLintSelection` record; migrate its library registration caller. |
| `cargo-octet/src/evidence_rails.rs` — `validate_kani_claims` | Three private positional flags | Pass the named `KaniClaimEvidence` record. |
| `cargo-octet/src/evidence_rails.rs` — `advisory_database_update` | Two private positional flags | Read the existing `EvaluationInput` at the receipt boundary. |
| `cargo-octet/src/architecture_ir/cargo_fixture_tests.rs` — `run_body_owner_compiler_fixture` | Two private positional flags | Remove the redundant wrapper and route callers through `BodyFixture`. |

The first compiled inventory on this base then reported:

| Site | Finding | Migration |
| --- | --- | --- |
| `cargo-octet/src/evidence_rails.rs` — `disables_advisory_fetch` | Two private positional flags | Pass the named `AdvisoryFetchEvidence` record. |

The second compiled inventory reported:

| Site | Finding | Migration |
| --- | --- | --- |
| `src/purity/effect_lints.rs` — `pure_contract_violation` | Three private positional flags | Pass the named `PureContractSignals` record. |

Each site was migrated rather than given an `allow` or an expectation. No
finding baseline, warning budget, or broad suppression was used.

The final compiled inventory used the local Dylint driver and library with both
lints denied:

```sh
RUSTC_WORKSPACE_WRAPPER=<ui-driver> \
DYLINT_LIBS='["<octet-lint-library>"]' \
DYLINT_NO_DEPS=1 \
DYLINT_RUSTFLAGS='-D raw_bool_parameter -D raw_bool_parameter_suppression' \
cargo check --workspace --all-targets --message-format=short
```

It exited **0** with zero `raw_bool_parameter` or suppression-guard lines.
Unrelated existing lint warnings remain. This is a check of the compiled
workspace targets under these two denied rules, not a claim that the whole
catalog passes a deny-all gate.

## Selected external consumer

No authorized external consumer worktree is part of this assignment. A sibling
`aspen` checkout exists in the workspace, but editing it or running its build
is outside this worktree's authority, so no consumer source migration,
compatibility change, or selected-target receipt is claimed. The consumer
migration task remains blocked until an owner-authorized consumer checkout is
provided.

## Same-callable Clippy co-report

`cargo-octet/tests/clippy_restriction_profiles.rs` compiles a disposable
four-`bool` public function under both rails. The Octet lint reports each
direct parameter, and the independent warning-level
`boolean-api-modeling` Clippy profile reports `fn_params_excessive_bools` in
its own receipt with checker `clippy`, profile `boolean-api-modeling`, and its
own finding family. The tools preserve separate identities and do not replace
one another.

## Upstream style boundary

The [source-bound Radiance style reference](boolean-api-modeling.md#upstream-style-boundary)
records the exact upstream revision, its MIT license, the guidance selected for
Octet, and the unrelated helper/panic guidance excluded from this rule. The
external style reference is not evidence for the compiled inventory or any
consumer's behavior.
