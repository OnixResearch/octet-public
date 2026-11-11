# Design: Promote Low-Noise Starter Lints

## Context

The repo now has two pieces of truth about the six new lint families:

1. archived classification evidence in
   `openspec/changes/archive/2026-04-16-new-lints-missing-principles/`
2. rollout docs that still keep all six families disabled in starter examples

That mismatch leaves the documented starter profile behind the evidence. The
next change should be narrow: promote only the four families that already met
our low-noise rule, prove the promoted starter profile on Aspen, and keep the
remaining two families clearly deferred.

## Goals / Non-Goals

**Goals:**
- promote only the four low-noise families into starter-profile guidance
- keep the two noisy families inventory-first in every copied starter example
- save a fresh Aspen replay for the promoted starter profile using the
  first-class consumer path
- make the docs show both the archived classification source and the current
  promoted starter decision

**Non-Goals:**
- altering lint semantics or the <=3 / <=5 classification rule
- changing the starter package scope
- promoting `unbounded_collection_growth` or `too_many_parameters`
- replacing the archived classification evidence from the previous change

## Decisions

### 1. Promote only families that already met the low-noise rule

**Choice:** Move exactly these families into starter-profile docs and examples:
- `explicit_defaults`
- `unbounded_channel`
- `usize_in_public_api`
- `negated_predicate`

Keep these families disabled and inventory-first:
- `unbounded_collection_growth`
- `too_many_parameters`

**Rationale:** The previous change already classified the six families using
saved Aspen evidence. Re-running the same decision logic should not reopen the
threshold question.

### 2. Save a new promoted-profile Aspen replay instead of relying only on the archived classification pass

**Choice:** Add a new replay helper and count checker under this change that run
Aspen's documented starter packages from the archived phase-1 reference
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`:
- `aspen-time`
- `aspen-hlc`
- `aspen-core`

The helper also runs a focused `aspen-coordination` spotcheck with that same
promoted lint set.

**Rationale:** The archived classification pass proved per-family counts. This
change needs a reviewable proof of the actual promoted starter profile that
future consumers will copy.

**Implementation:** Use a disposable Aspen worktree plus sibling symlinks for
`aspen-dns` and `aspen-wasm-plugin`, build `cargo-tigerstyle` from the Nix
package, write a self-describing consumer `dylint.toml`, and save:
- promoted starter default transcript
- promoted `aspen-coordination` transcript
- applied config snapshot
- count/assertion summary

The helper and checker must make the proof self-describing:
- each transcript prints the exact applied `dylint.toml` before the run
- the meta/check output states the enabled promoted families and the starter
  package set
- the checker asserts every printed count, including zero-finding claims

The checker contract for this change is exact:
- starter scope asserts `explicit_defaults = 1`, `unbounded_channel = 0`,
  `usize_in_public_api = 3`, `negated_predicate = 0`
- `aspen-coordination` asserts `explicit_defaults = 2`,
  `unbounded_channel = 0`, `usize_in_public_api = 0`,
  `negated_predicate = 2`
- the config snapshot and checker output must both show
  `unbounded_collection_growth` and `too_many_parameters` still disabled

### 3. Documentation must distinguish promoted starter families from still-pending families

**Choice:** Update the rollout kit so starter-profile examples enable the four
promoted families by omission from `disabled_lints`, while inventory examples
and high-noise guidance still list the two deferred families explicitly.

The exact copied starter-profile config shape for this change is:

```toml
[tigerstyle]
disabled_lints = [
  "function_length",
  "acronym_style",
  "numeric_units",
  "unbounded_loop",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "unbounded_collection_growth",
  "too_many_parameters",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_unwrap",
  "no_panic",
  "unjustified_no_panic_allow",
  "no_recursion",
]
```

That starter profile therefore enables these seven families by default:
- `ambient_clock`
- `compound_assertion`
- `contradictory_time`
- `explicit_defaults`
- `unbounded_channel`
- `usize_in_public_api`
- `negated_predicate`

**Rationale:** Consumers copy the examples. The examples must encode the actual
starter contract, not a stale "all six are pending" state.

This change does not alter source-attr guidance. The promoted families continue
to use bare lint names in `dylint.toml` (`explicit_defaults`, not
`tigerstyle::explicit_defaults`), and no new `register_tool` precondition is
introduced by this promotion.

## Verification Strategy

- `bash openspec/changes/promote-low-noise-starter-lints/evidence/replay-aspen-promoted-starter.sh`
- saved evidence artifacts:
  - `evidence/aspen-promoted-starter-default.txt`
  - `evidence/aspen-promoted-starter-coordination.txt`
  - `evidence/aspen-promoted-starter-config.toml`
  - `evidence/aspen-promoted-starter-counts.txt`
  - `evidence/check-aspen-promoted-starter.py`
- docs updated so starter/boundary-first examples no longer disable the four
  promoted families, while still disabling `unbounded_collection_growth` and
  `too_many_parameters`

## Risks / Trade-offs

**[Docs outrun reality]** Aspen may change after the promotion proof.
**Mitigation:** save a fresh replay in this change and keep the package scope
narrow to the documented starter profile.

**[Consumers over-promote]** Readers may assume the last two families are now
safe too.
**Mitigation:** keep the deferred pair explicit in starter examples, inventory
examples, and the high-noise guidance.
