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

Identity-verified Mantle, ChaosControl, or Molten observations can be linked with repeated `--external-report PATH`. Reports must bind the exact artifact, selected profile, reviewed SpaceWasm cohort, canonical report identity, recorded-only role, result, and no-claim-transfer boundary. Runtime success never changes a static denial.

## Corpora

`corpus.ncl` binds exact generated Wasm and report bytes, tools, seeds, inputs, transforms, expected issues, and shrink predicates.

The fast lane covers admitted and denied boundaries. The deep lane records deterministic wasm-smith and wasm-mutate outputs.

These outputs are validation evidence, not proof.

## Non-claims

A pass does not prove SpaceWasm decode, instantiation, execution, memory safety, semantic equivalence, sandbox effectiveness, configured runtime capacity, flight qualification, production readiness, certification, or release eligibility.
