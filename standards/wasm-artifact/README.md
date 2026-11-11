# Wasm/component artifact rail

Octet validates exact WebAssembly module or component bytes without execution.

The rail owns static fact extraction and policy decisions. Mantle and runtime consumers retain build, sandbox, capability, execution, and release authority.

## Profile and cohort

`profiles.ncl` is the typed source of truth and `generated/profiles.json` is the checked runtime input. The profile binds the reviewed wasm-tools `0.253.0` cohort with a BLAKE3 identity. Cohort changes require replay of the positive and negative corpus. Cohort drift, a recorded cohort identity that no longer matches its members, is a hard registry error that stops analysis before any artifact decision; profiles have no severity setting for it. The cohort must also name the `wasmparser` and `wasm-encoder` versions that the runner was built with, so a registry that omits either crate or claims another version fails validation even when its identity matches its own members. The runner also rejects a JSON registry with an unknown field at any level, a repeated entry in an allow/require list, an `allowed_worlds` entry that is not a `namespace:package/world[@semver]` name, `severity.*` values outside `advisory`, `warning`, and `error`, or a `duplicate_custom_sections` value other than `allow` or `reject`. The Nickel contract applies the same unique-list and world-name checks at export time.

## Standalone invocation

```console
cargo octet evidence collect \
  --rail wasm-artifact \
  --input module.wasm \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile portable-component-baseline \
  --artifact-dir artifacts/wasm-artifact
cargo octet artifact verify --artifact-dir artifacts/wasm-artifact
```

The bundle includes `wasm-artifact.bin`, exact-byte and normalized-interface identities, extracted facts, ordered JSON findings, a SARIF projection, profile/cohort identities, provenance, and explicit non-claims. `wasm-artifact.bin` holds the exact bounded bytes that were analyzed, and the receipt's config identity is the identity of the registry bytes that were analyzed. The runner reads `--input` and `--config` once each, so a pipe such as `/dev/stdin` works. The SARIF projection reports `advisory` findings at SARIF level `note` and includes a `region.byteOffset` only when the finding has a byte offset. Verification recomputes the normalized-interface identity from the recorded facts and rejects a receipt whose stored identity differs. It also rejects a receipt whose config identity differs from its recorded `registry_identity`, and recomputes the receipt status from the recorded findings with the rule collection uses: diagnostics or any `error` finding mean `failed`, other findings mean `manual_review_required`, and no findings mean `passed`. A relabelled status or a changed finding severity therefore fails verification.

## Molten effect surface

`molten-component-effects-v1` is separate from both unchanged baseline policies.
It requires exactly the root `preserves-payload` equality type import resolving
to `list<u8>`, the root `effect-request: func(list<u8>) ->
result<list<u8>, string>` import, and the sole `invoke` export with that same
function signature. The type import is canonical-ABI bookkeeping, not another
host function. `component_interface = "structural-v1"` resolves validated types;
type indices, debug strings, parameter labels, and producer metadata do not
authorize a signature. `required_imports` complements the existing required
exports. Unsupported structural types do not match this profile.

This is a shape check, not authentication of the Molten package/world name.
Canonical Preserves validation, effect interpretation, capability decisions,
actor semantics, and execution remain Molten-owned. No WASI or extra product
functions are admitted. The source-owned `fixtures/positive/component-effects.wat`
is an admitted static identity seed, not an actor implementation or runtime test.

The profile reuses the published `call-indirect-overlong` token for LLVM's padded
LEB table-zero immediate, alongside wasm1/component-model/bulk-memory. This does
not enable general reference types, SIMD, or all Wasm2. Neither existing
published baseline policy is changed by this effects profile.

Custom-section identities include their binary scope (`scope[N]::name`), where
`N` is the deterministic parser traversal ordinal. Separate modules/components
may each contain `name`; repeated `name` in the same scope is still rejected.
Producer sections remain recorded metadata, not provenance authority.

The external surface contains root exports and unresolved imports. Validation
must prove a local module/component instantiation before its bound imports can
leave that surface; they remain scope-qualified in `facts.bound_imports`.
Unused or exported reusable definitions retain unresolved authority. Nested
unresolved component imports are not authorized by root component allowlists.
Imported/aliased generative instantiations without a local implementation fail
resource accounting closed, rather than silently contributing zero resources.

Resource counts multiply each local definition by its enclosing instantiations
(saturating on overflow); unused definitions conservatively count once. Bound
memory/table/function imports are aliases, not new independent resources.
Canonical lowerings and resource intrinsics define core functions and consume
the function budget; canonical lifts and aliases do not add core definitions.
Component exports advance their module/component index spaces while retaining
the original definition identity; export-only instances do not advance them.
Definition limits still apply to every retained memory/table declaration; bound
import constraints remain in the recorded import facts. The reviewed real guest
has one memory and **two** independent tables: guest 5/5 elements and adapter 1/1;
the adapter's imported table is not a third table. This profile allows two tables
and otherwise retains baseline static limits. Missing declared maxima are
recorded as absent, not invented runtime bounds. Counts and declarations do not
prove allocation, effective runtime limits, or execution capacity.

Interface identities use the `octet-wasm-interface/v2` domain for the corrected
boundary semantics. Exact artifact BLAKE3 identity still hashes the original
bytes; profile identities still hash the selected canonical Rust profile DTO.
Omitted optional additions preserve the existing baseline profile identities.
The full registry identity changes when the new profile is added.

Regenerate the checked projection (then run focused checks separately):

```console
nickel export --format json standards/wasm-artifact/profiles.ncl > standards/wasm-artifact/generated/profiles.json
```

## SpaceWasm profile

`spacewasm-mvp` narrows the same exact-byte core to a reviewed NASA SpaceWasm cohort, hostless `run` ABI, WebAssembly 1.0 feature posture, and named static resource limits. See [`spacewasm/README.md`](spacewasm/README.md) for the pinned revision, source discrepancies, corpus, external-report contract, and non-claims.

## Mantle handoff

Mantle can supply the exact final bytes through the same command. It can also add a `mantle:` parent artifact identity.

Octet records a `mantle-exact-byte` handoff role. A `chaoscontrol:` or `molten:` parent identity without a `mantle:` identity records `external-orchestrator`. Other `--parent-artifact` values stay provenance links and leave the role `standalone`. Verification rejects a receipt whose handoff role does not match its recorded parent identities. Octet does not treat store paths, build success, runtime reports, or release decisions as static artifact facts.

## Claim boundary

A passing report establishes only bounded syntax, profile, identity, and source-shape facts. It does not prove execution correctness, semantic equivalence, sandbox effectiveness, capability authority, provenance truth, release eligibility, or whole-system safety.
