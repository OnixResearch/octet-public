# AeneasVerif integration roadmap

This roadmap assigns AeneasVerif execution, evidence, build, and lifecycle authority across the Onix stack.

It records planned ownership. It does not report proof, translation, build, migration, or release success.

## Current state

Octet has a pinned Aeneas, Charon, and Lean proof provider. It also has a recorded-only Charon semantic-analysis rail.

Valence validates the current Aeneas translation chain. Trellis has an active exact-source cross-verifier pilot for `abs_diff`.

The shipped Octet policy registry still selects an Octet-owned `safe_add` fixture. A reusable downstream consumer contract remains active Cairn work.

Octet has no Eurydice or Scylla execution rail.

## Ownership

| Surface | Owner | Authority |
|---|---|---|
| Charon and Aeneas execution | Octet | Provider policy, bounded execution, theorem admission, and receipts |
| Consumer theorem campaign | Consumer repository | Source selection, theorem intent, requirement mapping, and local claim |
| Evidence identity and linkage | Valence | BLAKE3 identity, roles, edges, and non-claims |
| Build and package observations | Mantle | Reproducible build inputs, outputs, and build evidence |
| Lifecycle decision | Repo-local Cairn | Change gates, readiness policy, and archived evidence |
| Exact artifact transport | Kamacite | Opaque identity transport without compiler authority |
| Differential runtime observations | ChaosControl | Bounded behavior comparisons without semantic-equivalence claims |

## Stage one: reusable Aeneas consumer contract

The immediate change is `.cairn/changes/generalize-aeneas-consumer-contract/`.

This change keeps provider pins in Octet. It lets a consumer select exact local source and maintained theorem files through reviewed policy artifacts.

The first external consumer is the Trellis `abs_diff` pilot. Trellis must use an immutable Octet revision and must not reconstruct provider pins.

The stage is complete only when Octet has a local external-consumer fixture and Trellis records a supported or exact blocked result.

## Stage two: Eurydice Rust-to-C export

Eurydice belongs in a separate optional Octet export rail. It must not share an Aeneas extraction contract unless exact compatibility is validated.

The rail must bind:

- exact Rust source and source selection
- the Eurydice-specific Charon revision and effective options
- LLBC and KaRaMeL inputs
- generated C bytes and compiler diagnostics
- panic and termination assumptions
- tool, target, resource, and non-claim identities
- positive and negative code-generation fixtures

For a proof-linked export, Valence can link a named Aeneas theorem receipt and a separate Eurydice generation receipt to the same Rust source.

The link does not transfer the theorem to the generated C. Mantle can build and package the C as a separate build artifact.

ChaosControl can add bounded Rust-versus-C differential observations. Those observations do not prove semantic equivalence.

Do not start this stage before the Trellis consumer contract produces stable evidence.

## Stage three: Scylla C-to-Rust migration

Scylla belongs in an experimental Octet migration rail for one named C corpus. It is not a default checked path.

The rail must bind:

- exact C source bytes and selected translation units
- Clang, Scylla, and KaRaMeL identities
- all source annotations and manual rewrites
- generated Rust bytes and diagnostics
- unsupported constructs and bounded failure classes
- positive and negative migration fixtures

Octet can admit the generated Rust through ordinary source and proof rails. That admission does not prove equivalence with the original C.

Valence can record C-to-Rust lineage after Octet publishes a concrete producer schema. Cairn can gate the named migration campaign.

ChaosControl can compare selected C and Rust behavior. Scylla remains experimental until a real corpus and bounded acceptance policy exist.

## Stable boundaries

Raw LLBC is a versioned provider artifact. It is not a stable stack-wide API.

Valence and Kamacite do not execute Charon, Aeneas, Eurydice, Scylla, Lean, Clang, or KaRaMeL.

Mantle does not own theorem admission or translation semantics. Cairn does not own verifier execution or evidence semantics.

A passing Aeneas receipt supports only its named theorem, exact generated model, source, assumptions, and toolchain.

## Upstream references

- [Charon](https://github.com/AeneasVerif/charon) extracts Rust compiler data into LLBC for analysis and verification tools.
- [Aeneas](https://github.com/AeneasVerif/aeneas) translates supported safe, sequential Rust models to proof assistants.
- [Eurydice](https://github.com/AeneasVerif/eurydice) translates a Rust subset through Charon and KaRaMeL to C.
- [Scylla](https://github.com/AeneasVerif/scylla) experimentally translates highly regular C through Clang and KaRaMeL to safe Rust.
