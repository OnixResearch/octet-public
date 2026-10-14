## Context

Tiger Style guidance still lacks lint coverage for six principles, but these
six do not all have the same implementation risk.

- `explicit_defaults` and `unbounded_channel` are mostly identity-matching
  problems once type resolution is available.
- `usize_in_public_api` and `too_many_parameters` are signature-shape checks.
- `negated_predicate` is a naming-pattern check with a well-defined false-
  positive boundary.
- `unbounded_collection_growth` is the highest-risk lint because broad alias or
  cross-block dataflow would create large false-positive surfaces.

The repo already has several hard-won constraints from napkin history:
- match compiler-known identities by resolved def path / lang items, not raw
  source strings
- avoid broad text scans when same-block structural reasoning is enough
- keep noisy or uncertain lints inventory-first until downstream evidence says
  otherwise
- add config keys through the existing repo-owned `Config` loader in
  `src/config.rs`, not via ad-hoc parsing

## Goals / Non-Goals

**Goals:**
- Add six warn-by-default lint IDs that cover missing Tiger Style principles
- Keep each lint scoped tightly enough to support deterministic UI fixtures
- Reuse existing config plumbing for `max_parameters`
- Produce Aspen evidence that records counts and classifies each new lint as
  starter-candidate or inventory-first
- Update the rollout kit, template, and README so consumers can see the saved
  Aspen classifications and keep non-promoted families out of starter examples

**Non-Goals:**
- Tool-lint namespace migration or group registration work
- Performance-domain lints
- Cross-function, alias-heavy, or interprocedural reasoning for
  `unbounded_collection_growth`
- Automatic starter-profile promotion during the same change

## Decisions

### 1. `explicit_defaults` matches resolved `Default` trait calls on foreign-crate types

**Choice:** Warn only when method resolution selects the `Default` trait's
`default` function and the resulting concrete type belongs to a different crate
from the current crate.

Sibling workspace crates count as foreign for this lint because they compile as
separate crate identities and can change defaults independently.

**Rationale:** The Tiger Style risk is relying on defaults outside the current
crate's review boundary. Matching by resolved trait identity avoids false
positives from unrelated inherent methods also named `default`.

**Implementation:** Late pass. Match `Default::default()`,
`<T as Default>::default()`, and `T::default()` only when the resolved callee is
`Default::default`. This includes user-written shorthand such as
`..Default::default()` and expanded attribute-driven forms like
`#[serde(default)]` when the lowered callsite still resolves to the same trait
function. Resolve the resulting type, require a concrete foreign ADT, and skip
current-crate types plus supported test contexts.

**Alternative:** Warn on every source form spelled `*.default()`. Rejected
because inherent constructors named `default` are not the same contract as the
trait default.

### 2. `unbounded_channel` matches resolved constructor identities, not just re-export spelling

**Choice:** Cover exactly three constructor families:
- `std::sync::mpsc::channel`
- `crossbeam::channel::unbounded`, resolved through the definition path
  `crossbeam_channel::channel::unbounded`
- Tokio's unbounded MPSC constructor, resolved through its internal definition
  path `tokio::sync::mpsc::unbounded::unbounded_channel`

**Rationale:** Napkin already proved Tokio re-exports do not always appear in
resolved def paths. The lint must match the actual definition identity.

**Implementation:** Late pass. Match call expressions against those resolved
paths and suggest the bounded alternative with explicit capacity.

**Alternative:** Match the source-path text `tokio::sync::mpsc::unbounded_channel`.
Rejected because it is fragile under re-exports and imports.

### 3. `unbounded_collection_growth` stays same-block and local-binding only

**Choice:** Warn only when all of these hold in one block:
- a local binding of type `Vec`, `HashMap`, `BTreeMap`, or `VecDeque` is
  initialized empty (`::new()` or empty `Default::default()`)
- that same binding is later grown via `.push()`, `.insert()`, or `.extend()`
  inside a `loop`, `while`, or `for`
- the same binding has no earlier `with_capacity`, `reserve`, `reserve_exact`,
  `try_reserve`, or explicit local len/capacity guard proving bounded growth

Anything alias-heavy or cross-block is skipped.

**Rationale:** This keeps the lint reviewable and implementable without a custom
alias-analysis engine. The rule catches the highest-signal cases while avoiding
"guessy" warnings.

**Implementation:** Late pass. Walk loop bodies, identify the mutated local
binding, then inspect the enclosing block for the binding's same-block creation
and prior reservation / guard statements. If creation, mutation target, or
bound proof is ambiguous, do not warn.

**Alternative:** Warn on any collection growth in loops unless the collection
was built with `with_capacity`. Rejected as too noisy and too broad.

### 4. `usize_in_public_api` targets inherent public APIs only

**Choice:** Warn on inherent `pub fn` parameter and return types that expose
`usize` or `isize`. Skip trait declarations and trait implementations where the
signature is dictated by the trait.

**Rationale:** Trait-directed signatures often cannot change. The Tiger Style
problem is consumer-facing inherent APIs that choose platform-sized integers
when fixed-width types would travel better across boundaries.

**Implementation:** Late pass over free functions and inherent methods with
public visibility. Exempt trait items and impls whose signature comes from the
trait.

### 5. `too_many_parameters` reuses the repo config path with `max_parameters`

**Choice:** Default threshold is 5 counted parameters, configurable via
`[tigerstyle].max_parameters` in `dylint.toml`.

Receivers (`self`, `&self`, `&mut self`) do not count.

**Rationale:** The repo already supports threshold-style config keys through the
serde-backed `Config` loader. Reusing that path is lower risk than inventing a
lint-local parser.

**Implementation:** Add `max_parameters` to `Config`, default it to 5, count
non-receiver parameters from the resolved signature, evaluate free functions,
inherent methods, and trait method signatures, and skip trait impls plus
supported test contexts. A HIR-aware pass is acceptable here because receiver
handling and test-context reuse are more important than pre-typeck timing.

### 6. `negated_predicate` uses whole-token prefixes on bool APIs and locals only

**Choice:** Restrict matching to boolean-returning functions and bool-typed
local bindings whose names begin with whole-token negative prefixes like
`is_not_`, `has_no_`, `has_not_`, `lacks_`, `cannot_`, `does_not_`, or `no_`.
Near-miss names such as `notify` do not match.

**Rationale:** The Tiger Style principle is about predicate wording, not any
identifier that happens to contain the letters `not`.

**Implementation:** Match identifier spelling with word-boundary semantics and
require boolean type / return type. A HIR-aware pass is acceptable here because
bool local bindings and function return types are both easy to inspect after
type checking. Keep a narrow exception for a private helper named exactly `not`.

## Verification strategy

- `explicit_defaults`: UI proof for foreign crate type, sibling workspace crate
  type, struct-update shorthand, and current-crate type; Aspen proof for
  attribute-driven shorthand like `#[serde(default)]`; test-context negative
  case.
- `unbounded_channel`: UI proof for `std`, `crossbeam`, and Tokio unbounded
  constructors plus bounded Tokio negative case.
- `unbounded_collection_growth`: UI proof for same-block `Vec::new()` + loop
  growth, prior `reserve(...)` suppression, explicit local bound suppression,
  and ambiguous outer-binding no-fire behavior.
- `usize_in_public_api`: UI proof for inherent public API warning, private
  function no-fire, and trait-directed signature no-fire.
- `too_many_parameters`: UI proof for default threshold, trait-signature
  coverage, and config fixture for `[tigerstyle].max_parameters`.
- `negated_predicate`: UI proof for negated bool function/local, positive form,
  near-miss `notify`, and non-bool identifier no-fire.
- Verification commands: `cargo test --lib`, `cargo test --test ui -- --ignored
  --nocapture`, and `bash openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/replay-aspen-new-lints.sh`.
- Aspen evidence artifacts for this change are:
  - `evidence/replay-aspen-new-lints.sh`
  - `evidence/check-aspen-new-lints.py`
  - `evidence/validation-counts.txt`
  - `evidence/aspen-<lint>-default.txt`
  - `evidence/aspen-<lint>-coordination.txt`
  - `evidence/config-<lint>.toml`
- Low-noise classification for this change means <= 3 findings on the Aspen
  starter scope and <= 5 findings on the focused `aspen-coordination` lib-only
  spotcheck. Anything larger stays inventory-first until a later promotion
  change revisits it.

## Risks / Trade-offs

**[Collection-growth false positives]** → same-block logic may still warn on a
loop whose effective bound is encoded indirectly. Mitigation: skip ambiguous
cases and require direct local evidence before warning.

**[External-default breadth]** → treating sibling workspace crates as external
increases warning surface. Mitigation: keep the rule warn-by-default and
inventory-first until Aspen evidence says it is low-noise.

**[Config surface growth]** → `max_parameters` adds another repo-owned config
key. Mitigation: reuse the tested `Config` loader pattern already used for other
numeric thresholds.
