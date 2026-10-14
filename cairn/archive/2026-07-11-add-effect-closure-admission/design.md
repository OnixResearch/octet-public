## Context

Octet's strict profiles already provide deny-level lint floors, reviewed core scopes, source-token function addresses, BLAKE3 profile/config identities, and artifact verification. Those facts are intentionally direct: they do not say that callees or dependencies preserve the caller's effect policy. The Valence pilot admitted two production shared-identity files under that bounded contract and exposed the next evidence boundary.

The new layer must remain ordinary `rustc`-compatible tooling. It must distinguish a conservative selected-source closure from a semantic compiler call graph, and an explicit dependency summary from proof of dependency behavior.

## Goals

- Compute a deterministic transitive effect closure over every function in selected `core_source_scopes`.
- Reject unknown source constructs, unresolved calls, stale function identities, forbidden role edges, disallowed effects, and dependency summaries that do not match the resolved Cargo/toolchain identity.
- Keep inference and closure logic pure and independently testable.
- Produce tamper-evident, replayable evidence with explicit non-claims.
- Preserve existing `cargo octet check` behavior and artifacts.

## Non-goals

- Complete Rust name resolution, monomorphization, macro expansion, trait dispatch, MIR semantics, runtime call ordering, or reachability.
- Proving dependency implementations match human-reviewed call summaries.
- Proving panic freedom, semantic determinism, compiler correctness, whole-program correctness, or release eligibility.
- Treating call summaries as silent waivers for opaque constructs.

## Decisions

### 1. Admission is a separate two-stage command

**Choice:** Add `cargo octet admit --profile deterministic-core --closure`. The shell first invokes the existing strict-profile engine into `<artifact-dir>/check/`. Closure analysis runs only when that phase exits successfully and produced deny-mode function-address evidence. V1 rejects a missing profile, a profile other than `deterministic-core`, or omission of `--closure`.

**Rationale:** A closure receipt must extend rather than replace the established lint/address admission. Keeping `check` unchanged preserves compatibility and prevents stronger evidence from being inferred from a weaker run.

### 2. The core consumes values; the shell owns effects

**Choice:** The pure core receives source envelopes, parsed Cargo/toolchain identities, function-address records, the embedded catalog, and the workspace policy as in-memory values. It returns a canonical report and diagnostics. The shell alone reads files, walks selected paths, invokes Cargo/Cargo metadata, writes artifacts, and renders output.

**Rationale:** Closure rules, fixed-point propagation, role validation, and receipt validation remain testable without filesystem/process mocks.

### 3. V1 inference is conservative source-token analysis

**Choice:** Parse selected Rust files with `syn`. Infer direct occurrences of `alloc`, `panic`, `unsafe`, `clock`, `random`, `fs`, `net`, `spawn`, `lock`, `ffi`, and `unknown`. Resolve an unambiguous selected local call by function name and path context. Assign stable call-site IDs from the caller function address, normalized call shape, and within-function occurrence index.

Opaque or ambiguous constructs become `unknown`; an exact call-site summary resolves only the call edge and never erases a direct effect. Every macro invocation remains `unknown` in v1 because `syn` does not expand or recursively analyze arbitrary macro tokens; known allocation or panic macros additionally retain their obvious effect. Conditional compilation, async/await, ambiguous trait or method dispatch, unresolved local names, and unsupported syntax also fail closed. `unsafe` blocks and foreign calls remain direct effects and cannot be erased by a call summary. Indexing, panic macros, `unwrap`/`expect`, and panic-capable arithmetic are conservatively classified as `panic`.

**Rationale:** A smaller conservative claim is reviewable now. The receipt labels its resolver and canonicalizer so a future HIR/MIR implementation can be a new evidence version rather than a silent semantic upgrade.

### 4. Profile allowances are explicit

**Choice:** The embedded catalog defines the complete effect vocabulary. V1 `deterministic-core` allows `alloc` but denies `panic`, `unsafe`, `clock`, `random`, `fs`, `net`, `spawn`, `lock`, `ffi`, and `unknown`. Diagnostics report both the direct source and every caller whose closure inherits a denied effect.

**Rationale:** Allocation can remain deterministic and is used by the Valence identity core; zero-allocation is a separate contract. Panic and ambient/platform effects violate this profile's closure goal.

### 5. Function addresses anchor nodes and call summaries

**Choice:** Every selected function must match exactly one deny-mode function-address record by normalized source path and function name. A node and each call-site ID bind that address. Missing, duplicate, stale, unsupported, or unmatched records deny admission.

**Rationale:** Source edits invalidate closure evidence and exact call summaries instead of inheriting an old admission by name alone.

### 6. Module roles and role edges are explicit

**Choice:** The Nickel workspace policy assigns every selected source path exactly one role from `core`, `shell`, `adapter`, `ffi`, or `proof`. The embedded catalog supplies the allowed role-edge matrix. A `deterministic-core` selected scope must be `core`; local selected calls must follow an admitted role edge. V1 does not infer a role from a path name.

**Rationale:** Explicit roles prevent ambient directory conventions or default fallbacks from widening the core/shell boundary.

### 7. External calls require exact, non-waiver summaries

**Choice:** Every non-macro call not resolved to one selected local function can be resolved only by a workspace-policy entry keyed by its stable call-site ID. V1 does not infer toolchain intrinsics from source names such as `Ok`, `Some`, or `Vec`; those calls require exact `core`/`alloc` provider summaries, while every macro remains directly `unknown`. The entry records a review label, provider kind, exact package/toolchain identity, target, sorted feature set, inferred effect set, and evidence reference. The shell requires an explicit Cargo `--target TARGET` argument rather than inferring the host or trusting ambient Cargo configuration, then resolves `cargo metadata --locked` for that target. Cargo package providers must match that graph. Toolchain providers must match the recorded rustc/target identity. Workspace summaries must match an exact workspace package identity.

Summaries cannot declare `unknown`, cannot resolve opaque non-call constructs, cannot change direct effects, and are reported as reviewed summaries rather than proof. A missing, extra, duplicate, broad, stale, or identity-mismatched summary denies admission.

**Rationale:** This makes dependency boundaries explicit and content-bound without pretending a review assertion verified dependency internals.

### 8. Closure uses a monotone fixed point and permits cycles

**Choice:** Sort nodes and edges canonically, initialize each node with its direct effects, and repeatedly union selected-callee effects into callers until no set changes. Bound iterations by the product of node and effect counts plus one; exceeding the derived bound is an internal consistency failure. Cycles and recursion are allowed as graph shapes and converge because the effect lattice is finite.

**Rationale:** Rejecting cycles would confuse graph shape with termination. A deterministic finite-lattice fixed point computes the bounded effect summary without runtime execution.

### 9. Admission artifacts are independently verifiable

**Choice:** Write:

- `<artifact-dir>/check/` — existing strict-profile bundle;
- `<artifact-dir>/command.txt` — normalized admission invocation;
- `<artifact-dir>/effect-closure.json` — canonical closure receipt and receipt identity;
- `<artifact-dir>/status.json` — admitted/denied status and parent identities;
- `<artifact-dir>/summary.txt` — deterministic human summary;
- `<artifact-dir>/provenance.jsonl` — artifact identities and bounded checked scope.

`cargo octet admit verify --artifact-dir PATH` reruns ordinary artifact verification over the complete strict-profile parent, recomputes closure BLAKE3 identities, validates canonical ordering/schema/non-claims, checks parent links, and returns invalid on tampering. It does not rerun source analysis, provide signer authenticity or authorization, resist coherent replacement of every trust root, or promote integrity into correctness.

**Rationale:** Reviewers can validate stored evidence without the original build process while retaining the limits of integrity verification.

### 10. Nickel owns human-authored policy

**Choice:** Add a typed Nickel catalog and workspace-policy contract, positive/negative fixtures, generated runtime catalog JSON, a copyable workspace template, registry classification, and deterministic freshness checks. Runtime Rust never executes Nickel.

**Rationale:** Effect vocabulary, profile allowances, roles, and reviewed call summaries are configuration/policy and benefit from contracts, deterministic export, and reviewable merge semantics.

## Risks / Trade-offs

- Source-token resolution intentionally rejects valid Rust patterns that a compiler-backed resolver could understand. Diagnostics and call-site IDs make those gaps reviewable; no permissive fallback is provided.
- Exact call-site summaries can be verbose and will churn when caller function addresses change. That churn is the intended stale-evidence behavior.
- Human-reviewed dependency effect summaries are bounded assertions, not dependency verification. Receipts preserve this non-claim and exact package/toolchain binding.
- Running the strict check before closure costs an additional phase but prevents closure-only evidence from bypassing established profile floors.
- Adding `syn` increases the runner dependency closure; package and standalone downstream checks must cover it.
