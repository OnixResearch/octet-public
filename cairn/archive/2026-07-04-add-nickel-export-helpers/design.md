## Context

Octet uses Nickel where humans compose policy or metadata before execution, but generated JSON/TOML/YAML artifacts are what most consumers and release gates should read. Several existing commands already export Nickel-owned data and validate freshness, but each family repeats command shape, manifest identity, and failure behavior.

The useful `rules_nickel` idea is intentionally small: a build rule names a Nickel source, dependencies, export format, and output. Octet should adapt the model as a Nix/Onix helper and optional CLI wrapper with stronger manifest and policy integration.

## Decisions

### 1. Keep the export interface small and typed

**Choice:** The export helper accepts a source Nickel file, dependency/import paths, optional field selector, optional contract path or contract expression, output format, output path, and config/receipt family ID.

**Rationale:** A small interface is easier to audit and reuse across policy registries, proof-rail manifests, dynamic profiles, and generated templates.

### 2. Manifest every generated output

**Choice:** Every helper-produced artifact records output format, output path, byte length, BLAKE3 content digest, source paths, source digests, dependency digests, Nickel binary identity when available, and exact command shape.

**Rationale:** Octet release evidence treats generated artifacts as reviewable bytes. The manifest prevents stale exports and mixed generations.

### 3. Prefer generation-time Nickel over runtime Nickel

**Choice:** Octet runner/runtime code consumes generated deterministic artifacts and does not evaluate Nickel during ordinary checks unless an explicit generation or validation command is selected.

**Rationale:** Consumers should not need Nickel installed just to run starter checks, and runtime behavior should be owned by Rust DTOs and deterministic generated inputs.

### 4. Preserve native tool config where appropriate

**Choice:** Nickel becomes the authored source only for config families that benefit from composition, contracts, or policy reuse. Native TOML/YAML/JSON remains the authored source where external tools require it and Nickel adds no validation value.

**Rationale:** Nickel should catch config errors before export, not obscure simple tool-native files.

## Risks / Trade-offs

- **Over-general helper:** A broad generator can become hard to reason about. Mitigate with a narrow typed interface and fixtures.
- **Secret leakage:** Generated manifests can accidentally index sensitive config. Mitigate with explicit non-candidate exclusions and secret-bearing fixture rejection.
- **Format differences:** Nickel export formatting can differ from existing hand-authored files. Mitigate with deterministic snapshot tests and migration only when intentional.
- **Runtime coupling:** Users may assume Nickel is required for all Octet checks. Mitigate by documenting generation-time vs runtime boundaries.

## Validation

- Repeated exports from the same inputs produce identical bytes and manifest entries.
- Missing or failing contracts reject the export before output is accepted.
- Stale output bytes or stale source digests fail manifest verification.
- Secret-bearing and unsupported-format fixtures fail closed.
- Cairn validate and gates pass for this change.
