# Bounded semantic merge verification rail

Status: bounded prototype with a bounded Rust frontend, an optional Lean-checking backend, and reviewed design.
This rail is opt-in and is not part of starter `cargo octet check`. It checks a supplied four-version
candidate; it never constructs, applies, or mutates a merge.

## Question and boundary

For an exact base `O`, variants `A` and `B`, and a candidate merge `M`, the
checked relation is per observable and per admitted input:

- if `A` changes the observation from `O`, `M` must match `A`;
- if `B` changes the observation from `O`, `M` must match `B`;
- if neither changes it, `M` must preserve `O`;
- if `A` and `B` change one observation incompatibly, the relation is
  unsatisfiable at that input.

The relation is a sufficient criterion for preservation. It can reject an
intentional integration, and it is not a reconstruction of developer intent.
It is defined here over a declared, finite input domain. A `verified` outcome
therefore means the relation held for the named profile, observables, and
admitted inputs; it is not a universal program proof.

## Admitted subset

The executable prototype models pure scalar functions over `i64` arguments with:

- integer literals and positional arguments;
- addition, subtraction, multiplication, division, remainder, and negation;
- conditionals where zero is false and any other value is true;
- direct calls to mapped logical items.

Arithmetic and abnormal outcomes are explicit:

- `checked`: overflow and division/remainder by zero produce an `overflow` or
  `divide-by-zero` abort observation;
- `wrapping`: two's-complement wrapping arithmetic; division/remainder by zero
  still abort.

Preconditions are conjunctions of per-argument integer comparisons. An
observable whose precondition admits no input yields `unknown`, never a
vacuous `verified`.

The memory and aliasing model is `pure-scalar`: no memory, pointers, mutation,
or aliasing. Dependency closure requires every reachable callee to have an
identical semantic identity in all four variants, unless the callee is itself a
declared observable participating in the four-version relation.

Excluded from this profile: unsafe code, raw pointers, FFI, concurrency,
opaque macros, dynamic or trait dispatch, external effects, floating point,
panics, nontermination, and exceptional control flow. A construct outside the
subset produces `unsupported`, not a semantic conflict.

## Item matching and source-to-IR correspondence

Every logical item has a role-tagged name mapping. Renames, signature changes,
changed dependencies, and missing items require an explicit reviewed mapping;
otherwise the outcome is `unsupported`. Textual correspondence, function
hashes, lint results, and Charon comparison rows are not merge evidence.

The prototype includes a bounded `syn`-based Rust frontend for the documented
scalar subset. The frontend maps every admitted function exactly once, rejects
unsupported constructs fail-closed, and emits a correspondence report binding
the exact source text identity, the emitted program identity, the function
inventory, the explicit obligation, and the frontend non-claims. The mapping is
syntactic; it does not prove Rust semantics, compiler correctness, or
whole-program behavior. Hand-authored variants remain valid inputs. Each
variant binds a source-text identity and a role-local semantic-subject
identity. Binding disagreement is `unsupported`; a permutation of the declared
role identities is reported as a role mismatch.

## Rust frontend

`octet_merge_rail::frontend::parse_source` accepts one Rust source file using
the admitted `i64` scalar subset:

- plain `fn` items with `i64` parameters and an `i64` return;
- integer literals with no suffix or `i64`;
- parameter references, add/sub/mul/div/rem, and integer negation;
- `if` expressions with both branches;
- direct calls to other admitted functions.

Local bindings, statements, semicolons, mutable or destructuring parameters,
generic items, method/field/index operations, unsafe blocks, macros,
references, closures, async/await, floats, non-`i64` types, and any other
construct outside the subset return a typed `FrontendError`; no partial program
is returned. The output exposes `into_variant`, `binding`, and
`correspondence` so a caller can build the four role-tagged variants and bind
their source/subject identities explicitly. The merge receipt's existing
`typed-scalar-ir` frontend identity is unchanged for hand-authored corpus
fixtures; the Rust frontend has its own profile identity and non-claims.

## Backend choice and trusted base

The default backend is a deterministic in-memory evaluator over the typed IR.
Its trusted base is the Rust implementation of the admitted integer operations,
the explicit arithmetic profile, the relation checker, and canonical BLAKE3
serialization. It has no solver, so `solver` is absent from receipts.

An opt-in `LeanBackend` first computes the expected observation with the
in-memory evaluator, then asks the pinned Lean kernel to check a concrete
theorem binding the encoded program result to that observation. The Lean
backend supports checked arithmetic only; wrapping is outside its declared
profile. Its trusted base is the Lean kernel, the fixed generated encoding, and
the pinned Lean executable. Evidence is bounded to the supplied concrete input:
it does not prove the relation for all inputs, the semantic adequacy of the
encoding, Rust compiler correctness, or whole-program behavior. A Lean
rejection, timeout, spawn failure, or nonzero exit is a tool error, never a
semantic conflict.

Direct four-program checking was chosen for the prototype. Shared-fragment
summaries and product construction are a later design option only if each
summary carries checked premises and exact dependency identities; otherwise the
rail must fall back to `unknown`. A symbolic-model proof, if added later, would
remain distinct from Rust extraction and runtime correspondence.

## Deterministic outcomes and receipts

Outcomes are `verified`, `counterexample`, `unknown`, `unsupported`, `timeout`,
and `tool-error`. A solver warning, evaluator exhaustion, or backend failure is
not a semantic conflict.

Each receipt binds the profile, frontend, model, verifier, backend, optional
solver, assumptions, role-tagged source and subject identities, item mappings,
observable domains, dependency identities, budgets, admission identity, exact
command, deterministic work report, outcome, optional Pijul provenance, and
consumer non-claims. Validation recomputes the receipt identity and rejects
stale or tampered receipts. Work is reported as deterministic evaluation and
relation-check counts, not wall-clock time.

Pijul native change identities are optional recorded-only provenance. Change
algebra, clean application, and equal trees provide no behavioral proof.

## Executed experiment

The reviewed corpus lives in `crates/octet-merge-rail/src/corpus.rs`; its
expected outcomes are authored independently of the checker. Run:

```sh
cargo run -p octet-merge-rail --bin octet-merge-rail-experiment
```

Observed results on this change:

| Case | Expected | Observed |
|---|---|---|
| identical | verified | verified |
| independent-changes | verified | verified |
| equivalent-edits | verified | verified |
| incompatible-changes | counterexample `incompatible@f` | counterexample `incompatible@f` |
| lost-changes | counterexample `lost-a-change@f` | counterexample `lost-a-change@f` |
| text-clean-semantic-interference | counterexample `interference@f` | counterexample `interference@f` |

Metrics: 6 cases, 6 expected matches, precision 6/6, coverage 6/6, outcome
counts `verified=3`, `counterexample=3`, `unknown=0`, `unsupported=0`,
`timeout=0`, and `tool-error=0`, and 80 deterministic work units.
These are observed counts for this corpus only; no paper performance claim and
no general precision estimate is made.

Negative controls in `crates/octet-merge-rail/tests/negative_controls.rs`
exercise missing observables, vacuous preconditions, changed callees, wrong
source bindings, swapped roles, stale receipts, tampered receipts, unsupported
constructs, domain overflow, interrupt budgets, injected backend failure, and
malformed role/binding inventories.

## Go/no-go decision

- **GO** for the bounded pure-core prototype, the bounded Rust frontend, the
  optional per-input Lean-checking backend, and the fixture experiment in this
  change, with the profile and non-claims above.
- **NO-GO** for production admission, universal verification, or any claim
  that the bounded frontend proves Rust semantic correspondence. A production
  frontend, a semantic-correspondence proof, and a checker with a stated trusted
  base and proof obligations must be separately scoped and reviewed. Until then, this rail may not gate merges or releases and may not
  relabel Charon comparisons, function hashes, lint results, or tests as merge
  proofs.

## Consumer non-claims

A verdict never provides automatic merge admission, source mutation,
whole-program correctness, semantic equivalence, developer-intent recovery,
release authority, or provider trust. Consumers must preserve the bounded claim
scope and all receipt non-claims.

## Implementation

- `crates/octet-merge-rail/src/model.rs`: typed four-version request and
  outcomes.
- `crates/octet-merge-rail/src/checker.rs`: canonical admission, mapping,
  bindings, dependency closure, bounded evaluation, and relation checking.
- `crates/octet-merge-rail/src/evaluate.rs`: pure scalar evaluator and injected
  backend contract.
- `crates/octet-merge-rail/src/receipt.rs`: deterministic receipt construction
  and validation.
- `crates/octet-merge-rail/src/corpus.rs`: reviewed fixture corpus and
  independent expectation table.
- `crates/octet-merge-rail/src/frontend.rs`: bounded `syn`-based Rust
  source-to-IR frontend, correspondence report, and fail-closed errors.
- `crates/octet-merge-rail/tests/frontend.rs`: frontend mapping, identity,
  rejection, admission, and binding-drift tests.
- `crates/octet-merge-rail/src/external_lean.rs`: opt-in pinned-Lean concrete
  verifier with a fixed checked-arithmetic encoding and bounded tool-error
  semantics.
- `crates/octet-merge-rail/tests/lean_backend.rs`: Lean success, overflow,
  rejection, timeout, wrapping-profile, and cycle tests.
