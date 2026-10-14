# Effect closure admission

`cargo octet admit --profile deterministic-core --closure` is a separate, opt-in evidence rail layered on the existing strict-profile check. It first runs ordinary `cargo octet check --profile deterministic-core` with the workspace's reviewed function-address baseline. Only a successful direct check can proceed to closure analysis.

<!-- r[impl octet.effect_closure.cli] -->
<!-- r[impl octet.effect_closure.boundaries] -->

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

Direct effects cannot be erased by a call summary. Every macro invocation remains `unknown` in v1 because `syn` does not expand or recursively analyze arbitrary macro tokens; known allocation or panic macros also retain their obvious effect. Conditional compilation, async/await, ambiguous calls, unsupported syntax, and other opaque shapes remain fail-closed. A future HIR/MIR resolver must use a new evidence version rather than silently broadening this claim.

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
- within-function occurrence;
- stable BLAKE3 call-site ID.

Review the actual source and resolved `dependency-graph.json`, then add one exact entry per accepted call. Each entry binds:

```nickel
{
  call_site_id = "b3:<caller-address-and-call-shape identity>",
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

The default artifact directory is `target/octet-admission`; `--artifact-dir` may select another location. A completed closure attempt writes:

- `check/` — the existing strict-profile parent bundle;
- `command.txt` — normalized admission invocation;
- `workspace-policy.json` — exact consumed policy bytes;
- `dependency-graph.json` — canonical resolved providers, target/toolchain, and lock identity;
- `effect-closure.json` — canonical nodes, calls, direct/closure effects, diagnostics, input identities, non-claims, and BLAKE3 receipt identity;
- `status.json` — admitted/denied phase status;
- `summary.txt` — deterministic operator summary;
- `provenance.jsonl` — artifact hashes and parent links.

Verify stored integrity without rerunning Cargo:

```bash
cargo octet admit verify --artifact-dir target/octet-admission --output-format json
```

Verification reruns the ordinary artifact verifier over the complete strict-profile parent, recomputes closure receipt and artifact identities, validates parent profile/config/function-address/source/policy/dependency linkage, regenerates status/summary/provenance expectations, and rejects tampering. This proves stored identity/linkage consistency only; unkeyed BLAKE3 receipts provide no signer authenticity, authorization, or defense against an actor who can coherently replace every artifact and trust root.

## Valence dogfood

<!-- r[verify octet.effect_closure.dogfood.initial] -->
<!-- r[verify octet.effect_closure.dogfood.final] -->

The first Valence run preserved a successful direct check followed by 78 unknown call edges over 16 selected shared-identity functions. Its reviewed Nickel policy binds those exact sites to the recorded `core`, `alloc`, `std`, `blake3`, and `serde_json` provider identities. The final run has zero closure diagnostics, permits only `alloc`, verifies independently, repeats byte-for-byte under unchanged inputs, and retains a tamper-negative report.

That result covers only the two selected Valence production files and exact recorded inputs. Broader `valence-core` and all dependency implementations remain outside the admission claim.
