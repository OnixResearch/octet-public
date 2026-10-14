# Content-addressed Rust evidence boundaries

This document fixes the Phase 7 boundary for Octet's Valence-oriented content-addressed Rust evidence work. It is a contract document, not an implementation claim for HIR/MIR extraction.

## Canonical v1 identity

`octet.function-object.v1` remains the canonical v1 review/provenance identity for selected Rust functions.

- The v1 object hash is a BLAKE3 identity over canonical source-token object content.
- The v1 object hash is evidence identity for review, replay, diffs, bundles, and name pointers.
- The v1 object hash is not a proof of behavioral correctness, semantic equivalence, build correctness, or distributed execution correctness.
- HIR, THIR, and MIR records MAY supplement a v1 object, but they do not replace the v1 object hash as the v1 review identity.

## HIR supplemental evidence boundary

Future HIR-derived evidence is toolchain scoped.

A future HIR evidence record MUST include:

- the referenced `octet.function-object.v1` object hash;
- exact rustc/toolchain identity;
- extractor name and version;
- HIR evidence schema version;
- resolved dependency summary where available;
- unresolved dependency, macro-origin, generated-code, and cfg caveats where applicable.

HIR evidence MAY improve dependency summaries and macro-origin reporting. It MUST NOT claim cross-toolchain stability unless a separate verifier proves that claim for the named toolchains and corpus.

The current bounded pilot command is:

```bash
cargo octet object supplement emit --family hir --function checked src/lib.rs --output-format json
cargo octet object supplement check --family hir --receipt hir-supplement.json --function checked src/lib.rs --output-format json
```

The pilot HIR supplement is source-token bounded. It reports contract status
`pilot-supported` only for the checked pilot emit/check surface and remains an
attached supplement to `octet.function-object.v1`.

The current rustc-backed HIR spike is narrower than general HIR support and is
selected-fixture-backed/toolchain-scoped. List the closed selector inventory
before choosing a shape:

```bash
cargo octet object supplement fixtures --family hir --spike rustc-hir-fixture --output-format json
```

The inventory reports each selected checked-in fixture shape, source path,
function name, default marker, replay commands, and explicit non-proof boundary
flags. Omit `--fixture-shape` to use the default checked arithmetic fixture, or
select one of the closed fixture shapes:

```bash
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --output-format json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --receipt hir-spike.json --output-format json
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --output-format json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --receipt hir-spike.json --output-format json
```

The rustc HIR fixture spike records `octet-rustc-hir-fixture-spike` extractor
identity, exact `rustc -vV`/host identity, selected fixture shape
(`checked-arithmetic` or `branch-control-flow`), fixture path/function context,
HIR observation hash, extraction caveats, and fail-closed replay comparisons. It
is not general HIR support; arbitrary-path and non-fixture HIR remain
planned/unsupported until their own positive and negative deterministic evidence
exists.

## MIR supplemental evidence boundary

Future MIR-derived evidence is compiler-version-coupled.

A future MIR evidence record MUST include:

- the referenced `octet.function-object.v1` object hash;
- exact rustc/toolchain identity;
- extractor name and version;
- MIR evidence schema version;
- summaries for call, panic, drop, async, unsafe, allocation, and effect surfaces only when the extractor can support them;
- explicit unknown/unsupported caveats for every summary family that is absent or incomplete.

MIR evidence MAY support stronger review, cache, or effect decisions. It MUST NOT claim stable identity across compiler versions, target triples, optimization modes, feature flags, or dependency graphs unless those dimensions are explicitly part of the compared evidence and a separate verifier proves equivalence.

The default MIR runtime seam remains fail-closed/non-fixture bounded until a
general extractor has positive evidence. The selected rustc MIR fixture contract
is narrower than general MIR support:

```bash
cargo octet object supplement fixtures --family mir --spike rustc-mir-fixture --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape checked-arithmetic --output-format json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape checked-arithmetic --receipt mir-spike.json --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape call-return --output-format json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape call-return --receipt mir-spike.json --output-format json
cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow --output-format json
cargo octet object supplement check --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow --receipt mir-spike.json --output-format json
```

That fixture-backed path MAY report `pilot-supported` only for
`checked-arithmetic` (`cargo-octet/tests/fixtures/mir-spike/checked.rs::checked`),
`call-return` (`cargo-octet/tests/fixtures/mir-spike/call_return.rs::call_then_return`),
and `branch-control-flow` (`cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs::choose_branch`).
It records `octet-rustc-mir-fixture-contract` extractor identity, exact
`rustc -vV`/host identity, fixture path/function/shape context, object-hash
linkage, `mir_fixture_contract_hash`, extraction caveats, and fail-closed replay
comparisons. Arbitrary-path MIR and non-selected fixture selectors remain
`unsupported-planned`.

The default THIR runtime seam remains fail-closed:

```bash
cargo octet object supplement emit --family thir --function checked src/lib.rs --output-format json
cargo octet object supplement check --family thir --receipt thir-supplement.json --function checked src/lib.rs --output-format json
```

Default THIR runtime receipts MUST keep status `unsupported-planned` and name
missing extractor support. A matching default THIR check proves only that the
same fail-closed receipt-critical fields replayed; it does not prove THIR
extraction support, correctness, semantic equivalence, pure-cache eligibility,
or cross-toolchain stability.

The selected THIR fixture contract is narrower than general THIR support:

```bash
cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture --output-format json
cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic --output-format json
cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic --receipt thir-spike.json --output-format json
cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape call-return --output-format json
cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape call-return --receipt thir-spike.json --output-format json
```

That fixture-backed path MAY report `pilot-supported` only for
`checked-arithmetic` (`cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`)
and `call-return` (`cargo-octet/tests/fixtures/thir-spike/call_return.rs::call_then_return`).
It records `octet-rustc-thir-fixture-contract` extractor identity, exact
`rustc -vV`/host identity, fixture path/function context, observation hash,
extraction caveats, and fail-closed replay comparisons. Arbitrary-path THIR and
any unimplemented non-pilot HIR/MIR/THIR family remain `unsupported-planned`.

## Downstream ownership boundary

Octet owns Rust extraction and object emission.

- Octet owns source scanning, function-object records, object hashes, object markers, name-pointer manifests, object diffs, caveat summaries, bundles, and replay metadata emitted by `cargo octet`.
- Trellis owns verified abstract admission models for definitions, dependency closures, names, recursive groups, and ability/effect admission. Trellis does not own Rust source extraction.
- Cairn owns lifecycle policy, gate execution, replay receipts, and receipt validation around Octet bundles. Cairn consumes Octet receipts without owning or redefining Rust extraction semantics.
- Crunch/Mantle may consume Octet object and bundle hashes as build/proof evidence identities. Octet hashes alone do not prove build correctness or binary reproducibility.
- Aspen may consume Octet object and bundle hashes as CI, forge, and runtime evidence identities. Octet hashes alone do not prove distributed execution correctness or live-system safety.

## Pilot boundary

Crunch/Mantle and Aspen pilots are bounded consumers of Octet evidence.

A pilot contract SHOULD name:

- selected source paths or crate profiles;
- Octet command and version;
- object/bundle schema versions;
- checker/config/source/executor hashes used by the consumer;
- caveats that block pure-cache or stronger admission;
- replay command and receipt path.

A pilot contract MUST keep correctness claims outside Octet object hashes unless a downstream verifier supplies separate proof evidence.

## Required negative language

When describing this layer, use bounded wording:

- accepted: "content-addressed review/provenance identity";
- accepted: "bounded local determinism receipt";
- accepted: "toolchain-scoped HIR/MIR supplemental evidence";
- rejected: "function hash proves correctness";
- rejected: "MIR hash is stable across rustc versions";
- rejected: "Octet owns Cairn lifecycle policy";
- rejected: "Octet hashes prove Aspen/Crunch execution correctness".
