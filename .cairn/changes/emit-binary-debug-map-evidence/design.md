# Design: Emit binary debug-map evidence

## Goal

Produce a deterministic observation that selected compiled symbol ranges map to selected source spans under an exact binary and debug artifact identity.

## Architecture

A std shell opens explicit files, enforces byte limits, and passes immutable byte slices plus validated profile data into a pure parser.

The parser returns normalized rows or deterministic issues. File access, process execution, Cargo invocation, environment access, and output writing remain outside the core.

## Dependencies

The producer uses:

- `object` for bounded object-file and section access;
- `gimli` for DWARF units, line programs, ranges, and split-debug data;
- `rustc-demangle` for display-only normalized Rust names.

The receipt binds exact crate versions and the Octet producer identity. Dependency use does not transfer parser or compiler correctness claims.

## Profile and limits

Nickel owns the `elf-dwarf-v1` profile. It declares admitted object class, architecture, DWARF forms, source-root mappings, required sections, and every resource limit.

Limits cover input bytes, sections, compilation units, symbols, ranges, line rows, frames, path bytes, and emitted records. Limit overflow fails closed.

The Rust receipt schema is authoritative for output. Generated Nickel contracts and positive and negative fixtures guard compatibility.

## Identity model

Each receipt binds:

- exact binary and optional debug-companion BLAKE3 identities;
- object format, architecture, build identity when present, and selected profile identity;
- raw symbol-name hash, display-only demangled name, address range, and mapped source span;
- optional Octet function address and typed Valence requirement reference;
- producer, toolchain, limits, truncation state, diagnostics, and non-claims.

Raw addresses remain binary-local. They are not stable cross-build function identities.

## Source path handling

DWARF can contain absolute build paths. The shell supplies an explicit profile-owned mapping from admitted build roots to repository-relative roots.

Unmapped absolute paths, parent traversal, non-UTF-8 paths, conflicting mappings, and paths outside the admitted source scope fail or remain excluded by profile.

Receipts emit only safe repository-relative paths and path hashes. They do not disclose private build roots.

## Linkage

A row links to an Octet function address only when source identity, relative path, and bounded span agree with a verified function-address artifact.

A demangled symbol-name match alone is insufficient. Duplicate or ambiguous candidate mappings fail closed in strict mode.

Typed requirement refs must validate against the supplied Valence and Cairn identities. Legacy text IDs remain recorded-only compatibility data.

## CLI and evidence rail

The rail is selected explicitly:

```text
cargo octet evidence collect --rail binary-debug-map --input <BINARY> --debug-companion <FILE> --config <PROFILE> --artifact-dir <DIR>
```

A companion is optional only when required DWARF data is present in the binary. Failed runs must not leave a successful receipt bundle.

## Verification

Positive fixtures include an admitted ELF with inline debug data and one with a split companion. Negative fixtures mutate one section, path, span, identity, or limit at a time.

Release handoff tests prove Valence role preservation and reject promotion to formal proof, behavioral correctness, or release eligibility.
