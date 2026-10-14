## Why

Octet already produces source-shape, provenance, Function Object, fingerprint, and Valence export evidence, but the checked Rust path does not yet require every function in a configured scope to have a stable content address. That leaves review and release evidence tied to paths, symbols, and manually selected objects instead of a complete content-addressed function inventory.

The stack needs an Octet-owned extraction rail that walks Rust source, canonicalizes function definitions, computes BLAKE3 addresses, and emits a portable receipt that Kamacite can preserve and Valence can wrap. Octet is the right producer because it owns Rust source-shape analysis and `cargo octet check`; downstream layers should not parse Rust to recover the function universe.

## What Changes

- Add an optional Rust function-address Octet rail with lint-like diagnostics and adoption modes such as `off`, `observe`, `warn`, and `deny`.
- Add scoped rollout controls so teams can content-address one crate, module, path set, or changed-function set before requiring the whole workspace.
- Define canonical function-address material with domain-separated BLAKE3 identities, source-envelope identities, spans, canonicalizer identity, local dependency edges when known, and explicit caveats for unsupported or generated code.
- Add fail-closed artifact verification in enforcing modes for missing functions, stale addresses, unsupported syntax without an exclusion, duplicate locator collisions, malformed receipts, and overclaims.
- Emit Kamacite-compatible function-address receipts and optional Valence adapter input from verified scopes, while allowing non-enforcing observe/warn modes to build the initial baseline.
- Document the boundary: Octet proves checked Rust function inventory/address evidence, not Rust semantic correctness, compiler correctness, whole-program safety, or release eligibility.

## Impact

- **Files**: new Octet function-address extraction, receipt, fixture, documentation, policy/registry, and Cairn lifecycle artifacts.
- **Testing**: positive and negative function-address fixtures, adoption-mode fixtures, artifact verification, export checks, policy freshness, focused Rust tests, `cargo octet check` or equivalent focused checks, and Cairn validation/gates before archive.
