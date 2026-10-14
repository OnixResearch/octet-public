# Proposal: Emit binary debug-map evidence

## Summary

Add an optional Octet evidence rail that derives bounded binary-to-source linkage from explicit ELF and DWARF bytes.

## Motivation

The stack can bind a release binary hash to producer sidecars and source function identities. It does not independently compare compiled debug metadata with those source identities.

A debug-map rail adds a second bounded observation. It can detect stale sidecars, wrong debug companions, missing source mappings, and symbol or span drift.

## Scope

- Add an `elf-dwarf-v1` binary debug-map profile and `binary-debug-map` evidence rail.
- Use pinned `object`, `gimli`, and `rustc-demangle` crates in a std evidence producer.
- Parse already-loaded binary and optional split-debug bytes through a pure bounded core.
- Link normalized symbol and source-span rows to Octet function addresses and typed Valence requirement refs.
- Emit Rust-owned receipts with generated Nickel contracts and explicit non-claims.

## Affected specs

- `evidence-provenance`: profile, extraction, linkage, failure behavior, and release handoff.

## Dependencies

The typed requirement link follows Valence change `adopt-content-bound-requirement-references`. The selected implementation must pin exact reviewed crate versions through Cargo and Nix.

## Non-goals

- Do not prove source-to-binary semantic equivalence or compiler correctness.
- Do not disassemble instructions or infer full control flow.
- Do not support PDB, every object format, or stripped binaries without an admitted debug companion in v1.
- Do not treat demangled symbol text as a canonical function identity.
- Do not make the rail part of starter `cargo octet check`.

## Completion evidence

The change is complete when positive ELF/DWARF fixtures pass, malformed and stale inputs fail closed, resource limits are policy-bound, and Valence handoff preserves the recorded-only claim.
