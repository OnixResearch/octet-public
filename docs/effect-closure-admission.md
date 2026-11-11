# Effect closure admission

`cargo octet admit --profile deterministic-core --closure` is a separate, opt-in evidence rail layered on the existing strict-profile check. It first runs ordinary `cargo octet check --profile deterministic-core` with the workspace's reviewed function-address baseline. Only a successful direct check can proceed to closure analysis.

<!-- r[impl octet.effect_closure.cli] -->
<!-- r[impl octet.effect_closure.boundaries] -->
<!-- r[verify octet.effect_closure.boundaries.non_claims] -->

Effect closure v1 source-parses the configured `core_source_scopes`, anchors each supported function to deny-mode source-token function-address evidence, classifies direct effects and call shapes, resolves selected local calls, applies exact reviewed summaries to every other supported call, and propagates effects to a finite fixed point.

It does not provide complete Rust name resolution, macro expansion, monomorphization, trait or dynamic-dispatch resolution, MIR call-graph completeness, dependency implementation verification, runtime reachability or ordering, panic freedom, semantic determinism, compiler correctness, verifier soundness, whole-program correctness, or release eligibility.

## Effect policy

<!-- r[impl octet.effect_closure.catalog] -->
<!-- r[impl octet.effect_closure.inference] -->

The Nickel source of truth is `standards/effect-closure/catalog.ncl`; runtime Rust embeds `standards/effect-closure/generated/effect-closure-catalog.json`. V1 uses this canonical vocabulary:

| Effect | Typical source-token evidence | `deterministic-core` closure |
| --- | --- | --- |
| `alloc` | allocating constructors, collection, clone/growth, owned-string conversion | allowed |
| `panic` | panic/assert macros, indexing, `unwrap`/`expect`, panic-capable arithmetic | denied |
| `unsafe` | unsafe function or block | denied |
| `clock` | ambient clock API | denied |
| `random` | ambient RNG/ID API | denied |
| `fs` | filesystem API | denied |
| `net` | network API | denied |
| `spawn` | thread/task spawn | denied |
| `lock` | lock acquisition | denied |
| `ffi` | foreign function boundary | denied |
| `unknown` | opaque or unresolved source shape | denied |

Direct effects cannot be erased by a call summary. Every macro invocation remains `unknown` in v1 because `syn` does not expand or recursively analyze arbitrary macro tokens; known allocation or panic macros also retain their obvious effect. Conditional compilation, async/await, ambiguous calls, unsupported syntax, and other opaque shapes remain fail-closed. Attributes are macro invocations too: any attribute on a function or one of its parameters, inside its body, or on an enclosing `impl`, trait, inline module, or file (`#![...]`) makes that function `unknown`, unless it is one of the inert built-in attributes `allow`, `warn`, `deny`, `forbid`, `expect`, `deprecated`, `doc` (including doc comments), `inline`, `cold`, `must_use`, `track_caller`, `repr`, `non_exhaustive`, `feature`, `no_std`, `recursion_limit`, or `register_tool`. Procedural attribute macros, `cfg`, `cfg_attr`, `test`, `bench`, and `derive` are all `unknown`, and so are tool attributes such as `#[rustfmt::skip]`, `#[clippy::msrv]`, or `#[diagnostic::on_unimplemented]`: a module, `use`, or `extern crate` named after the tool, or a Cargo dependency renamed to it, shadows the built-in tool, and the recorded dependency graph does not carry dependency renames. Tool paths inside lint attributes, such as `#[allow(clippy::pedantic)]`, stay inert. A future HIR/MIR resolver must use a new evidence version rather than silently broadening this claim.

## Workspace policy

Copy `templates/effect-closure-policy.ncl` and the contract in `standards/effect-closure/contracts.ncl`, then export the reviewed policy to the workspace root:

```bash
nickel typecheck effect-closure-policy.ncl
nickel export --format json effect-closure-policy.ncl > octet-effect-closure.json
```

The runtime does not execute Nickel. Admission also requires an explicit Cargo `--target TARGET` argument after the command delimiter. Octet refuses to infer a host target because ambient Cargo configuration could make the direct check and recorded dependency graph disagree.

The workspace JSON must contain:

- schema `octet-effect-closure-workspace-policy/v1`;
- profile `deterministic-core`;
- the workspace-relative reviewed function-address baseline;
- exactly one explicit `core`, `shell`, `adapter`, `ffi`, or `proof` role for every selected source path;
- sorted exact call-site summaries;
- explicit non-claims.

All paths selected by `deterministic-core` must have role `core` in v1. The catalog admits `core -> core`; no path-name convention or missing-role default is used.

## Review unknown calls

<!-- r[impl octet.effect_closure.dependency_summaries] -->

Start with no call summaries. V1 grants no name-based intrinsic exemption: apparent prelude/toolchain calls such as `Ok`, `Err`, `Some`, or `Vec::new` still require exact `core` or `alloc` provider summaries because source tokens cannot prove that those names were not shadowed. Macro calls remain directly `unknown` and cannot be summarized.

A successful direct check followed by denied closure writes `effect-closure.json`; each `call_edge_unknown` includes:

- caller source path and function address through its containing node;
- normalized call shape;
- resolution scope: the caller's module and what each leading name of the call path resolves to outside the caller;
- within-function occurrence;
- stable BLAKE3 call-site ID.

Occurrences count a function's calls in syntax order. The pattern of a `let`, `if let`, `while let`, or `for`, including a `let` type annotation, comes before its initializer.

The call-site ID (domain `octet-effect-closure-call-site/v2`) binds the caller's function address, source path, resolution scope, call shape, and occurrence. The resolution scope reads like `module=self; crate::util::tidy=src/lib.rs util: mod util, src/util.rs dep_a: extern, src/util.rs tidy: use dep_a::tidy`: the caller's inline-module path within its file, then, for each path the call references (the call path, or for a `<Type as Trait>::` path the type's and the trait's paths; generic arguments dropped, leading-`::` paths left out), the sorted, deduplicated set of declarations it resolves through, each labelled with its source path, inline-module chain, and the name it declares. A path resolves through, in order, the `use` bindings and items of the blocks lexically enclosing the call (innermost first; blocks around a nested `fn` item count, but a `use` in a sibling block or inside a nested item does not, and glob `use`s of enclosing blocks are followed alongside), the enclosing `impl` or trait header (for `Self` and its generic parameters), and the declarations of the caller's module: `use` bindings and items (functions and type-like items by kind and name; type aliases, constants, statics, extern crates, and `macro_rules!` definitions by their tokens; non-doc attributes such as `cfg` included). A call inside a module item declared in the function body is `unknown` and cannot be summarized. An undeclared name resolves to `prelude` (`undeclared` after a module path, `extern` at the head of a `use` path), together with the module's wildcards when it has any: glob `use`s, item macro invocations (bound by their tokens and, for a macro the module defines, by that `macro_rules!` definition's tokens), and items carrying an attribute macro or `derive` (any non-inert attribute other than `cfg`, bound by the BLAKE3 hash of the item's tokens), since each may expand to or import an item of any name. After a module path written in the call or in an import (`self::gen::tidy`), a name the path goes through that the module does not declare is `unknown` and cannot be summarized when the module has an item macro or attributed item that could generate it; a candidate probed through a glob `use` (`Vec::new` looked up through `use crate::util::*`) binds that module's wildcards instead.

Local paths are followed to the declaration they finally name: `crate::`, `super::`, and `self::` prefixes, module declarations, `use` bindings and re-exports whose paths are local, and glob `use`s (`use super::*`). A `use` path whose first segment no declaration of its module provides (other than an import of the same name, as in `use log::log;`) may name an extern crate or a name a glob `use` of the module brings in, so both are bound: `use crate::*; use util::tidy;` follows `util` through the crate root. A path that reaches a selected free function or inherent method as its only target is a local edge, like a single-segment call in the caller's own module; otherwise the scope binds each declaration on the way, so a reviewed summary for `crate::util::tidy()` goes stale when `util` re-exports another `tidy`. An `Owner::method` path through a local type binds the type's declaration with its attributes and, when no selected inherent method of that name exists in the type's module, the module's wildcards, since a derive, an attribute macro on an `impl`, or an item macro may generate the method. Each lookup of a name in a module runs once per referenced path, so a glob cycle (a child's `use super::*` with the parent's `pub use child::*`) adds nothing new and a prelude call there stays summarizable, and the scope grows with the distinct declarations reached. A path that needs more than 1024 lookup steps or 32 nested hops is `unknown`. Following stops at what the selected sources cannot prove: an inline module is always followed, but `crate::`, `super::` out of a file module, and out-of-line `mod name;` declarations are followed only between modules with a proven crate position, reachable from a selected `src/lib.rs`, `src/main.rs`, or `src/bin` root through module declarations without `path`, `cfg`, or macro attributes, in Cargo's conventional file layout. A call whose path reaches an unproven or unselected module this way is `unknown` and cannot be summarized; select the target module and its crate root, or call it through a dependency path.

A method call's scope reads like `module=self; methods=prelude, src/core.rs trait: use crate::ext::PersistExt, src/ext.rs PersistExt: use fs_ext::PersistExt, ...; header=impl Store`: every declaration that may bring the called method's trait into scope, each resolved like a call path. That is the `use` bindings of the blocks enclosing the call, then each `use` binding, trait item, and wildcard of the caller's module, with local imports followed to the declaration they name and local glob imports (`use crate::ext::*`) followed to the same method sources of their module, plus the enclosing `impl` or trait header, whose bounds may supply methods. Each module's method sources are resolved once per admission run, each `use` binding with its own lookup bound, so a module with hundreds of imports stays within bounds. A method call whose module imports from an unproven or unselected local module is `unknown` and cannot be summarized. Editing any `use` or trait item of a module, or of a module it imports from, therefore re-keys every method call site in that module. Macro calls bind only the module. A reviewed summary therefore goes stale, and its call becomes `unknown`, when a declaration its call path resolves through changes, or when the same function moves to or is copied into another file or module, while edits to declarations of other names keep its ID. Summaries reviewed under call-site domain v1, or under earlier v2 scope rules, must be re-reviewed against the IDs the next denied receipt reports.

Review the actual source and resolved `dependency-graph.json`, then add one exact entry per accepted call. Each entry binds:

```nickel
{
  call_site_id = "b3:<call-site ID reported by the denied receipt>",
  review_label = "what source/API was reviewed",
  callee = "reviewed callee label",
  provider = {
    kind = "cargo-package", # or toolchain/workspace
    name = "package-or-toolchain-surface",
    version = "exact resolved version",
    source = "exact registry/workspace/toolchain source",
    target = "exact target triple",
    features = ["sorted", "resolved", "features"],
  },
  effects = ["alloc"],
  evidence = {
    kind = "cargo-lock-review-v1",
    locator = "Cargo.lock#package",
    blake3 = "b3:<reviewed lock or provider identity>",
  },
}
```

Cargo/workspace summaries must match an exact provider in `cargo metadata --locked` and bind the recorded lockfile identity. Toolchain summaries must match the exact rustc/target provider identity. Missing, duplicate, unused, broad, unknown-effect, stale, target/feature/version/source-mismatched, or evidence-mismatched summaries deny admission. A summary is a bounded reviewed assertion; it is not proof that dependency implementation behavior matches the declared effect set.

## Fixed-point closure

<!-- r[impl octet.effect_closure.transitive] -->
<!-- r[verify octet.effect_closure.transitive.cycle] -->

A call is a selected local edge only when its source path proves the target: a single-segment call such as `helper()` names a free function in the caller's file and inline module, `Owner::name()` names a method of an inherent `impl Owner` block in that same scope, and `self.name()` inside an inherent method names a method of that same inherent owner in that scope. Qualified module paths such as `crate::shell::now()`, qualified-self paths such as `<std::fs::File>::create()` or `<File as Write>::write()`, leading-`::` paths such as `::helper()`, trait default and trait-impl methods, `self` calls from trait bodies, and calls whose proven target is not unique remain unresolved and need an exact summary or stay `unknown`. A qualified-self call keeps its `<Type>` or `<Type as Trait>` prefix in its call shape.

A path whose first segment is a name bound inside the caller never resolves locally, because that binding may shadow the same-named module item. Bound names are collected across the whole function: parameters and other `let`, `for`, `match`, and closure pattern bindings, generic parameters of the function and of its enclosing `impl` or trait, and `use` declarations and items inside the body. A glob `use` inside the body disables local resolution for that function. For example, `helper()` in `fn root(helper: fn()) { helper(); }` and `Owner::name()` in `fn root<Owner: Tick>() { Owner::name() }` stay unresolved even when a module-level `helper` or inherent `Owner::name` exists.

A path used as a value rather than called, such as `std::process::abort` in `run(std::process::abort)`, is a `value:` edge. It resolves locally under the same rules as a call, so `run(helper)` adds `helper`'s effects to the caller; otherwise it needs an exact summary or stays `unknown`. This covers constants, enum variants, and associated constants used as values, such as `None` or `u32::MAX`. Paths in compile-time positions are not edges: patterns, types, array repeat lengths such as `LEN` in `[0u8; LEN]`, braced const generic arguments such as `make::<{ LEN }>()`, associated-const generic arguments, const parameter defaults, enum discriminants, and the initializers of `const` and `static` items and associated consts declared in the body, such as `const X: usize = LEN;`. Calls in those positions are still recorded. Reads of `self` or of a local that is in lexical scope where it is read are not edges either. Parameters and const generic parameters are in scope in the whole body. A `let` binding is in scope from the next statement to the end of its block. `match`, `if let`, and `while let` bindings are in scope only in their arm, guard, or body. A `for` pattern is in scope in the loop body, and closure parameters in the closure body. The const generic parameters of a nested `impl` or trait are in scope in all of its items, so `N` in `impl<const N: usize> B<N> { fn len() -> usize { N } }` is not an edge. A nested item does not see the enclosing function's locals, and an item or `use` declared in a block shadows outer locals of the same name. So in `if false { let cleanup = 1; } x.map(cleanup)`, the `cleanup` passed to `map` is still an edge.

Closure nodes use the function-address producer's item model: one node per outermost `fn` item. Items nested inside a function body, such as an inner `fn`, are not separate nodes; their tokens are scanned as part of the enclosing function, and a call never resolves to a nested item as a selected local edge.

Selected local calls form a canonically ordered graph. Each node begins with direct inferred and reviewed-summary effects. Octet repeatedly unions selected callee effects into callers until no finite effect set changes. Recursive cycles are retained and converge; this graph fact does not claim termination or runtime reachability.

Admission succeeds only when there are no policy, address, role, edge, provider, inference, or denied-effect diagnostics.

## Artifacts and verification

<!-- r[impl octet.effect_closure.evidence] -->

Run admission with an explicit target:

```bash
cargo octet admit --profile deterministic-core --closure \
  --artifact-dir target/octet-admission \
  -- --target x86_64-unknown-linux-gnu
```

The default artifact directory is `target/octet-admission`; `--artifact-dir` may select another location. Both `admit` and `admit verify` resolve a relative `--artifact-dir` against the workspace root, not the current directory. The directory must be missing, empty, or an earlier admission bundle; Octet refuses any other non-empty directory before deleting anything. A completed closure attempt writes:

- `check/` — the existing strict-profile parent bundle;
- `command.txt` — normalized admission invocation;
- `workspace-policy.json` — exact consumed policy bytes;
- `dependency-graph.json` — canonical resolved providers, target/toolchain, and lock identity;
- `effect-closure.json` — schema `octet-effect-closure-receipt/v2`: canonical nodes, calls (each with its resolution scope), direct/closure effects, diagnostics, input identities, non-claims, and BLAKE3 receipt identity;
- `status.json` — admitted/denied phase status;
- `summary.txt` — deterministic operator summary;
- `provenance.jsonl` — artifact hashes and parent links.

Verify stored integrity without rerunning Cargo:

```bash
cargo octet admit verify --artifact-dir target/octet-admission --output-format json
```

Verification reruns the ordinary artifact verifier over the complete strict-profile parent, recomputes closure receipt and artifact identities, validates parent profile/config/function-address/source/policy/dependency linkage, regenerates status/summary/provenance expectations, and rejects tampering. For an `admitted` receipt it also requires the node set to equal the records in `check/function-addresses.json`, each reviewed-summary edge to equal its `workspace-policy.json` call summary, and every policy call summary to be used. It also requires each selected source to have exactly one `core` role in `workspace-policy.json`, every node to have role `core`, and every local edge to follow the catalog's role-edge matrix. Each reviewed summary's evidence must still match the receipt's recorded Cargo.lock identity (Cargo and workspace providers) or its provider identity (toolchain providers), as admission requires. A denied receipt therefore cannot be relabelled admitted. `cargo octet artifact verify` applies the same checks to the closure artifacts of an architecture admission run. This proves stored identity/linkage consistency only; unkeyed BLAKE3 receipts provide no signer authenticity, authorization, or defense against an actor who can coherently replace every artifact and trust root.

Both verifiers read the receipt's `schema_version` before decoding anything else. A receipt under any other schema, including `octet-effect-closure-receipt/v1` from before call edges carried their resolution scope, is reported as an unsupported effect-closure receipt schema; rerun admission to produce a v2 receipt.

## Kamacite effect handoff

<!-- r[impl octet.effect_closure.kamacite_mapping] -->
<!-- r[impl octet.effect_closure.kamacite_dispositions] -->
<!-- r[impl octet.effect_closure.kamacite_comparison] -->
<!-- r[impl octet.effect_closure.kamacite_cli] -->
<!-- r[impl octet.effect_closure.kamacite_handoff] -->
<!-- r[impl octet.effect_closure.kamacite_boundary] -->
<!-- r[verify octet.effect_closure.kamacite_fixtures] -->

`cargo octet admit --profile deterministic-core --closure --kamacite-effect-profile PATH` is an optional extension that runs only after the direct check and effect closure are admitted. Without the option, arguments, artifacts, receipts, exit status, and claim boundaries are unchanged.

The Nickel source of truth is `standards/effect-closure/kamacite-profile.ncl`, with contracts in `standards/effect-closure/kamacite-contracts.ncl` and generated runtime input at `standards/effect-closure/generated/kamacite-effect-profile.json`:

```bash
nickel typecheck standards/effect-closure/kamacite-profile.ncl
nickel export --format json standards/effect-closure/kamacite-profile.ncl \
  > standards/effect-closure/generated/kamacite-effect-profile.json
```

The profile binds one frozen Kamacite effect-summary compatibility projection by exact identity and file BLAKE3. `standards/effect-closure/fixtures/kamacite/effect-summary.json` is a frozen local fixture, not a live Kamacite cohort. Octet never invokes Kamacite and never reimplements its Preserves canonicalization.

Ambient effect dispositions stay separate from semantic emissions. V1 permits only `alloc` as `profile-local`; every other ambient effect, including `clock`, `fs`, `net`, and `unknown`, remains `denied` and cannot be renamed or discharged by an emission rule. A profile that marks any other effect profile-local fails validation.

Each semantic-emission rule binds one exact function address, an optional exact reviewed call-site identity, one exact provider identity, one exact Kamacite `effect-operation` identity, an evidence reference, and explicit non-claims. Rules that repeat a site, name a stale identity, use a broad unresolved scope, or point at a non-reviewed call fail closed.

The comparison emits one of these classes:

| Class | Admitted | Meaning |
| --- | --- | --- |
| `emitted-and-declared` | yes | the exact emission operation appears in the closed Kamacite summary |
| `profile-local` | yes | an ambient `alloc` effect remains visible without becoming an operation |
| `declared-not-emitted` | yes | the closed summary declares an operation that selected Octet evidence does not emit; this is not a dead-code or coverage claim |
| `emitted-not-declared` | no | an exact emission names an operation absent from the closed summary |
| `stale-identity` | no | a profile, provider, function, call-site, or summary identity does not match its parent |
| `ambiguous-emission` | no | one site matches multiple rules, a scope is unresolved, or a summary repeats an operation |
| `unknown` | no | closure is not admitted, an ambient effect is denied or unmapped, or a supplied artifact is invalid |

On success, admission writes `kamacite-effect-profile.json`, `kamacite-effect-summary.json`, `kamacite-effect-comparison.json`, and `kamacite-effect-handoff.json`. The handoff binds the closure receipt identity, canonical mapping-profile identity, projection identity, Kamacite effect-summary and operation identities, comparison rows, producer role, diagnostics, and non-claims. `cargo octet admit verify` re-derives every Kamacite artifact from its stored parents and rejects tampering, including a re-identified handoff whose parent identity changed.

A frozen cohort update is an explicit reviewed change: regenerate the projection, update the profile's exact identities and `projection_blake3`, and rerun the positive and negative fixtures. Stale identities fail closed until that update lands.

A passing comparison records bounded agreement between selected Octet source-token evidence and the supplied frozen Kamacite projection. It does not prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility. Octet does not own canonical Kamacite semantics, and the handoff grants no consumer, publication, or runtime authority.

## Valence dogfood

<!-- r[verify octet.effect_closure.dogfood] -->
<!-- r[verify octet.effect_closure.dogfood.initial] -->
<!-- r[verify octet.effect_closure.dogfood.final] -->

The first Valence run preserved a successful direct check followed by 78 unknown call edges over 16 selected shared-identity functions. Its reviewed Nickel policy binds those exact sites to the recorded `core`, `alloc`, `std`, `blake3`, and `serde_json` provider identities. The final run has zero closure diagnostics, permits only `alloc`, verifies independently, repeats byte-for-byte under unchanged inputs, and retains a tamper-negative report.

That result covers only the two selected Valence production files and exact recorded inputs. Broader `valence-core` and all dependency implementations remain outside the admission claim.
