# Binary debug-map rail

Octet derives bounded ELF/DWARF binary-to-source linkage from exact supplied bytes
without executing, disassembling, or rebuilding anything.

The rail owns normalized symbol, address-range, and source-span rows plus exact
artifact identity. It does not own source semantics, compiler correctness,
requirement satisfaction, release eligibility, or any Valence claim promotion.

## Profile

`profiles.ncl` is the typed source of truth and `profiles.json` is the checked
runtime input. The `elf-dwarf-v1` profile binds:

- admitted object format (`elf`), architecture (`x86_64` or `aarch64`), and debug
  format (`dwarf`);
- required DWARF sections (`.debug_abbrev`, `.debug_info`, `.debug_line`);
- explicit `source_roots` mappings from absolute build roots to
  repository-relative roots;
- `unmapped_path_policy` (`fail` or `exclude`) and `strict_linkage`;
- `companion_match = "build-id"`;
- every resource limit: binary bytes, debug bytes, sections, compilation units,
  symbols, ranges, line rows, frames, path bytes, and emitted rows;
- per-family severities, the `recorded_only` role, and explicit non-claims.

The profile identity is the BLAKE3 hash of the selected canonical profile DTO.
The receipt's registry identity is the BLAKE3 hash of the exact `--config` bytes
that were parsed. A registry with an unknown field, a missing limit, an
unsupported format, an empty or conflicting source-root mapping, or a promoted
role fails before any binary parsing.

## Standalone invocation

```console
cargo octet evidence collect \
  --rail binary-debug-map \
  --input target/release/example \
  --debug-companion target/release/example.debug \
  --config standards/binary-debug-map/profiles.json \
  --profile elf-dwarf-v1 \
  --function-address-artifact artifacts/function-addresses.json \
  --requirement-refs artifacts/requirement-refs.json \
  --artifact-dir artifacts/binary-debug-map
cargo octet artifact verify --artifact-dir artifacts/binary-debug-map
```

`--debug-companion`, `--function-address-artifact`, and `--requirement-refs` are
optional. The shell opens each supplied file through its parent-directory
capability, applies the profile byte limit before parsing, and rejects `--run`;
it never invokes a compiler, linker, or disassembler. The receipt, summary, and
provenance sidecars are written atomically and the receipt is published last, so
an interrupted collection never leaves a bundle that claims success.

## Inline and split debug data

An inline binary must carry the required DWARF sections. A stripped binary must
be supplied with an exact split-debug companion whose `.note.gnu.build-id`
matches the binary. A companion that is missing, lacks a build identity, or
carries a different build identity fails with `wrong-companion` before any row is
published. Compressed debug sections are not admitted in v1; supply uncompressed
DWARF.

## Source paths and privacy

DWARF absolute build paths are normalized only through an explicit profile
source-root mapping. Unmapped absolute paths fail (`fail`) or are excluded
without a finding (`exclude`), parent traversal and non-UTF-8 paths fail, and
over-long paths hit the path-byte limit. Receipts record only canonical
repository-relative paths and their BLAKE3 path hashes; absolute build roots never
enter a finding message, row, or bundle artifact.

## Linkage

A row links to an Octet function address only when the function-address
artifact's repository path matches, its source name matches the DWARF source
name, and its recorded line/column falls inside the DWARF line span for the same
address range. A demangled symbol-name match alone never selects a candidate.
Duplicate in-span candidates fail with `ambiguous-function`; a same-symbol
candidate outside the DWARF span fails with `stale-span`. Strict linkage is
required by `elf-dwarf-v1`.

Typed requirement references are supplied separately and bind repository,
revision, requirement-registry identity, and requirement id to a function
address. Legacy text requirement ids remain recorded-only compatibility data.
The handoff validator admits only `recorded_only`; formal-proof,
semantic-equivalence, requirement-satisfaction, behavioral, and release roles are
rejected.

## Resource limits and truncation

Every limit overflow stops analysis with a `resource-limit` finding and names the
exceeded limit in `truncation`. A truncated receipt is never reported as
complete evidence and never passes receipt validation. The shell rejects
oversized binary or debug bytes before parsing.

## Non-claims

- Binary debug-map evidence records bounded ELF/DWARF linkage only and does not
  prove source-to-binary semantic equivalence, compiler correctness, or
  behavioral correctness.
- Demangled symbol text is display metadata and is not a canonical function
  identity.
- Raw binary-local addresses are not stable cross-build function identities.
- Binary debug-map evidence is optional and not part of starter
  `cargo octet check`.
- Binary debug-map evidence does not prove requirement satisfaction or release
  eligibility.
