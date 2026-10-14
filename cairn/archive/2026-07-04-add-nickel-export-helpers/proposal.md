## Why

Octet already relies on Nickel for human-authored standards policy, proof-rail metadata, dynamic evidence profiles, and selected templates. Each export path currently has to define its own shape, command, freshness rules, and artifact manifest behavior. The `rules_nickel` repository provides a useful minimal abstraction: declare a Nickel source, dependencies, export format, and output file, then let the build produce deterministic tool input.

Octet should adapt that abstraction for the Onix/Nix workflow so Nickel-authored configuration can be exported, hashed, and checked consistently without making runtime `cargo octet check` depend on Nickel evaluation.

## What Changes

- Add a repo-owned Nickel export helper contract for source file, import dependencies, optional field selection, contract to apply, output format, and output path.
- Provide Nix/flake wiring or a small Octet command surface that evaluates Nickel deterministically and writes JSON, TOML, YAML, or raw output as required by the downstream tool.
- Emit and verify a BLAKE3-backed export manifest that records source paths, dependency paths, Nickel binary identity, output format, output path, byte length, and content digest.
- Reuse the helper for new Nickel-authored config and migrate existing ad hoc exports only when doing so reduces duplication without breaking compatibility.
- Document when Nickel is appropriate and when a tool-native format should remain the authored source.

## Non-Goals

- Do not add Bazel as an Octet build requirement.
- Do not replace tool-native configuration formats when the tool must consume them directly and humans do not need Nickel composition.
- Do not make `cargo octet check` evaluate Nickel dynamically in ordinary consumer runs.
- Do not allow secret-bearing config exports to enter generated artifact manifests.

## Impact

- **Files**: Nix/flake helper or CLI wrapper, standards policy/export code, generated artifact manifest handling, docs, fixtures, and Cairn specs.
- **Behavior**: Nickel-authored config families gain one deterministic export and freshness pattern instead of bespoke commands.
- **Testing**: stable export snapshots, stale output rejection, invalid contract rejection, unsupported format rejection, no-secret fixture rejection, and Cairn validation/gates.
