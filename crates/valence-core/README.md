# Octet Valence legacy fixture

This directory preserves the final Octet-hosted Valence core as a frozen compatibility fixture. Its package identity is `octet-valence-legacy-fixture`; it is excluded from the Octet workspace, is not publishable, and is not a canonical `valence-core` provider.

Octet production code depends on the exact standalone Valence revision recorded in `standards/valence-core-cutover/manifest.ncl` and delegates through `cargo-octet/src/valence_core_shim.rs`. The fixture remains only for bounded drift and compatibility checks covering Function Object v1, evidence-index, and evidence-codebase-index behavior.

## Boundary

This fixture MUST NOT be selected by Octet or downstream consumers as Valence implementation code. It does not own CLI parsing, filesystem access, source scanning, publication, release evidence, or standalone Valence semantics.

Compatibility checks prove selected source ownership and fixture behavior only. They do not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, release eligibility, or whole-program safety.
