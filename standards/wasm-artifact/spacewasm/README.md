# SpaceWasm MVP artifact profile

`spacewasm-mvp` is an execution-free Octet profile over exact WebAssembly core-module bytes.

It is a restrictive, hostless `run`-export ABI review profile. It is not a universal SpaceWasm ABI. It does not execute or embed SpaceWasm.

## Reviewed cohort

- Repository: `https://github.com/nasa/spacewasm`
- Git revision: `e24cf09355a90497148eb5029fdb8e3400bd63e3`
- Derived cohort identity: `b3:7c537eef7bff66d0a90508715de08de79c9aaac23352e1007a6d47e7fb4674a2`
- Upstream `Cargo.lock` identity: `b3:4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1`
- Reviewed sources: `Cargo.lock`, `README.md`, `REQUIREMENTS.md`, `src/SPEC.md`, `src/code.rs`, `src/compiler.rs`, `src/opcode.rs`, and `src/types.rs`

The upstream Git revision is a Git SHA-1 interoperability identifier. Octet derives profile, projection, artifact, interface, report, and cohort identities with BLAKE3.

`support-projection.ncl` records the reviewed support boundary. Its checked JSON export is embedded in the runtime profile. The review found that `REQUIREMENTS.md` permits selected post-MVP proposals in principle while the README matrix and reviewed decoder expose WebAssembly 1.0 plus mutable globals. Octet therefore admits only wasmparser's `WASM1` feature set and fails closed on the broader proposals.

## Static profile

The profile requires:

- A core module, not a component, WIT, or WASI artifact.
- No imports.
- Only allowed exports, with `module::run` required.
- No start function or `memory.grow`.
- At most one memory and one table.
- Named limits for artifact size, functions, pages, table elements, parameters, and local words.
- Recorded-only producer metadata.
- Deterministic rejection of duplicate sections.

The profile limits are intentionally no wider than the reviewed upstream support limits. Runtime allocator pages, code pages, control frames, stack depth, fuel, and host implementations remain external configuration and cannot be inferred from module declarations.

## Standalone and Mantle handoff

```console
cargo octet evidence collect \
  --rail wasm-artifact \
  --input module.wasm \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile spacewasm-mvp \
  --artifact-dir artifacts/spacewasm
cargo octet artifact verify --artifact-dir artifacts/spacewasm
```

Mantle can supply exact final bytes and a `mantle:` parent artifact.

Octet independently measures the bytes and records the handoff. It does not import store-path, build, materialization, sandbox, or release claims.

## Optional runtime observations

Identity-verified Mantle, ChaosControl, or Molten observations can be linked with repeated `--external-report PATH`. Reports must bind the exact artifact, selected profile, reviewed SpaceWasm cohort, canonical report identity, recorded-only role, result, and no-claim-transfer boundary. Only a report that passes every check is recorded in `wasm_artifact.external_evidence`. A report that fails any check is not recorded; each failed check becomes an `external-evidence-mismatch` finding at the profile's declared `external_evidence` severity, so an `advisory` or `warning` severity leaves the receipt reviewable instead of failed. Each such finding names the report's 1-based `--external-report` position and, when the report parses, its claimed producer and `report_identity`. Runtime success never changes a static denial.

## Corpora

`corpus.ncl` binds exact generated Wasm and report bytes, tools, seeds, inputs, transforms, expected issues, and shrink predicates.

The fast lane covers admitted and denied boundaries. The deep lane records deterministic wasm-smith and wasm-mutate outputs.

These outputs are validation evidence, not proof.

## Molten pilot producer contract

The additive `spacewasm-molten-abi-pilot` selects the actual
`molten.wasm.abi.v1` surface: exported wasm32 `memory` with an explicit maximum
of at most 256 pages, `molten_alloc(i32)->i32`,
`molten_dealloc(i32,i32)->()`, and `molten_run(i32,i32)->i64`.
Only `molten:hostcall::assert` and `molten:hostcall::send`, each
`(i32,i32)->i64`, are statically allowable. The runtime consumer must additionally
require the exact per-manifest declared subset. No start, memory growth, tables,
WASI, components, or post-MVP proposals are admitted. At least one memory page
is required, but static shape does not prove the allocator or descriptor protocol.

`spacewasm-mvp-host-pilot` is a separate ChaosControl diagnostic profile:
`molten.observe(i32)->()`, `molten.pause()->()`, `run()->()`, and mutable i32
`total`. It does not replace the Molten ABI profile or prove continuation
behavior. Both pilot profiles admit function imports by name and resolved
signature; the core type index is immaterial.

The persisted Rust-owned `evidence-rail-receipt.json` has schema
`octet-evidence-rail-receipt/v1`; its `wasm_artifact` field binds exact artifact,
registry, profile, wasm-tools cohort, runtime source/cohort/support projection,
static facts, and ordered findings. `wasm_artifact.facts.function_abis` contains
resolved `direction`, `namespace`, `name`, `parameters`, and `results`, not just
export kinds or function-type indices. The internal
`octet-wasm-artifact-analysis/v1` envelope is not the persisted bundle contract.
Hostless `spacewasm-mvp` now checks the exact `run()->()` signature too.
Oversized bytes are denied before parser/validator construction.

Required operator smoke, after building the selected Octet source:

```console
cargo octet evidence collect --rail wasm-artifact --input "$MOLTEN_MODULE" --config standards/wasm-artifact/generated/profiles.json --profile spacewasm-molten-abi-pilot --artifact-dir "$RECEIPTS/molten-static"
cargo octet artifact verify --artifact-dir "$RECEIPTS/molten-static"
```

For the real producer positive/negative and exact-byte tamper smoke, run:
`python3 standards/wasm-artifact/spacewasm/smoke.py /absolute/cargo-octet /absolute/wasm-tools`.
This executes the supplied binaries; it is not a source-text or mock test.
The matching executable release is `wasm-tools 1.253.0`; its component crate
cohort is `0.253.0` (the profile calls that cohort `wasm-tools-0.253.0`).
Do not attempt to install a nonexistent `wasm-tools` CLI release `0.253.0`.
Retain the measured executable paths, versions, and hashes with the receipts.

The native change's `evidence/replay.py` retains independently collected actor
and diagnostic receipts outside the checkout and exercises valid-Wasm signature
mutations, stale cohort input, and forged external evidence. These checked-in
fixture receipts are static producer verification, not final Mantle workload
receipts or proof that Molten has consumed them.


Use exact final Mantle module bytes, not WAT source, a store-path name, or a
hand-authored receipt. The operator must generate the receipt from real inputs;
no synthetic passing receipt is supplied here. Pin the public Octet source
revision of the actual producer and retain its toolchain/provenance bundle.
Module producer custom sections are recorded input, not authenticated producer
or compiler identity. Mantle owns actual build/toolchain facts; ChaosControl
owns runtime observations. All use the reviewed NASA revision above, not an
invented release tag or an alternate engine standing in for SpaceWasm.

Replay `fixtures/positive/molten-abi.wat` and `host-segments.wat` after converting
with the pinned wasm-tools cohort. Negative smoke must include no memory maximum,
wrong allocator/host signature, forbidden hostcall, component/WASI/proposal inputs,
stale cohort, changed module bytes, and forged external evidence. Existing corpus
and Rust tests cover these boundary classes; new focused tests add signature
resolution, bounded pre-parse rejection, and both pilot ABI variants.


## Non-claims

A pass does not prove SpaceWasm decode, instantiation, execution, memory safety, semantic equivalence, sandbox effectiveness, configured runtime capacity, flight qualification, production readiness, certification, or release eligibility.
