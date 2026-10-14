## Context

Octet already has the annotation-driven purity workflow OxCaml's `[@zero_alloc]` article advocates — for effects other than allocation. `src/purity/effect_lints.rs::pure_contract_violation` checks a banned-effect list gated on the recognized pure-function marker, and the accepted `purity-effect-lints` spec carries the honest disclaimer that the marker "enforces direct source-shape checks rather than transitive whole-program purity proof." `src/performance/allocation_in_loop.rs` already has a type-aware allocator classifier (`supported_allocating_type`, matching `alloc`/`std` def paths for `Vec`/`String`/`Box`/`HashMap`/etc.) with a clean functional core (`classify_allocation_in_loop` / `AllocationInLoopFacts`), but it scopes the check to loop bodies only. Allocation is the one effect class not wired into the purity contract.

## Decisions

### 1. Allocation becomes a first-class `EffectClass` variant

**Choice:** Add `EffectClass::Allocation` and `PureViolation::Allocation`, and add allocation to the banned-effect checklist the `pure_attr_contract` and new `zero_alloc_attr_contract` lints share.

**Rationale:** One classifier, one vocabulary. Reusing the existing `allocation_in_loop.rs` allocator table avoids a second copy of the `alloc`/`std` def-path knowledge and keeps the loop lint and the contract lint consistent when a new allocator type appears.

### 2. A dedicated `zero_alloc_attr_contract` lint, not a flag on `pure_attr_contract`

**Choice:** Add a separate `zero_alloc_attr_contract` lint gated on `#[tigerstyle::zero_alloc]` / `#[octet::zero_alloc]` / `#[zero_alloc]`. Do not silently fold allocation into `pure_attr_contract`, because `#[pure]` callers today may reasonably allocate.

**Rationale:** `[@zero_alloc]` in OxCaml is opt-in per function. Folding allocation into the existing `#[pure]` marker would be a behavior change for every current `#[pure]` user. A separate marker and lint keeps adoption opt-in and the default level independently configurable, matching how `ambient_env`, `ambient_random`, and the `*_in_core` lints each own one effect family.

### 3. Direct source-shape checks only; no transitive call-tree proof

**Choice:** The lint checks direct allocation sites in the annotated function body. It does not chase the call tree.

**Rationale:** OxCaml's `[@zero_alloc]` is a compiler-level whole-call-tree analysis. Octet is a Dylint late pass doing intraprocedural snippet/type checks. Claiming parity would be dishonest. The accepted `pure_attr_contract` spec already carries the "direct source-shape checks rather than transitive whole-program purity proof" disclaimer; the new lint reuses that wording verbatim. If a maintainer needs true transitive non-allocation proof, that belongs in the `verified/` + Verus proof-rails infrastructure, not in a heuristic lint.

### 4. The `[@zero_alloc assume]` escape hatch maps onto `unjustified_allow`

**Choice:** Any `#[allow(tigerstyle::zero_alloc_attr_contract)]` (or bare `#[allow(zero_alloc_attr_contract)]`) MUST be paired with a justification, enforced by the existing `structure/unjustified_allow.rs` lint. There is no `assume`-style "lie" attribute that silently disables the check.

**Rationale:** OxCaml's `[@zero_alloc assume]` lets you assert that a wrapper is zero-alloc without proof. The valuable part of that pattern is that the opt-out is *named and visible*. Octet already has the machinery for that — `unjustified_allow` — so reusing it keeps every escape hatch auditable in the diff rather than inventing a parallel mechanism. Silent escapes are exactly what the project's tigerstyle posture rejects.

### 5. `local_` / `stack_` is out of scope

**Choice:** Do not add an OxCaml `local_`/`stack_` analog for "value does not escape, stack-allocate it."

**Rationale:** Rust's type system already decides stack-vs-heap (`Box`/`Vec`/`String` = heap; arrays, tuples, structs of stack fields = stack). Clippy already covers the "unnecessary heap" cases (`unnecessary_box`, `box_collection`, `boxed_local`). The "value does not escape" half overlaps with the existing `resource_return_from_core` lint. Low novelty, high overlap, not worth the complexity.

## Risks / Trade-offs

- `Vec::new()` and friends are pervasive; a `zero_alloc` lint will be noisy unless adopted deliberately per hot path. Mitigation: default level `warn` (matching `pure_attr_contract`), opt-in marker, and the existing `allow`-with-justification escape hatch.
- Macro expansion (`vec![]`, `format!`) can hide allocation constructors behind non-`Call` HIR shapes. The classifier must inspect the resolved type of the bound value as well as the call shape, mirroring how `allocation_in_loop.rs::expr_supported_allocation_type` already uses `cx.typeck_results().expr_ty`.
- Reusing the `allocation_in_loop.rs` classifier means the two lints share false-positive and false-negative profiles. Any tightening of the shared classifier must update both UI fixtures.

## Validation

Add positive and negative UI fixtures, classifier unit tests in `effect_lints.rs`, and catalog freshness coverage (README, lint reference, metadata, fixture manifests, default-level snapshots, downstream rollout guidance). Run `cargo test --lib`, the ignored UI rail, autofix rail if any machine-applicable suggestion is added, Cairn validation, and Cairn gates.
