# Design: Valence Function Object core crate seam

## Boundary

`valence-core` owns pure deterministic Function Object v1 construction. Its API accepts already-discovered source-scanner facts: function name, normalized function source, dependency-set hash, dependency identity status, macro-call flag, source path, source span, tool name, and tool version. It returns serializable Function Object v1 DTOs with canonical object hashes, caveat summaries, pure-cache eligibility metadata, and explicit no-correctness claims.

`cargo-octet` remains the shell and scanner. It still discovers Rust files, reads source, calls `fingerprint::find_functions`, computes source-scanner dependency facts, normalizes paths, handles object markers, renders corpus/bundle/supplement reports, prints output, and exits with existing statuses.

## Compatibility

The extracted core must preserve the current Function Object v1 canonical hash payload and visible DTO field names. Existing object corpus fixtures and aggregate Valence gates should continue to validate after fixture refresh only if intentional output drift occurs. This slice is expected to preserve object hashes because the hash algorithm and payload stay byte-compatible.

## Verification

Verification must demonstrate:

- `valence-core` tests cover stable Function Object v1 hashing and fail-closed caveat/non-purity metadata.
- focused `cargo-octet` object tests still pass through the shell/scanner seam.
- current evidence-index command behavior still passes after the additional crate dependency changes.
- named Nix gates `object-corpus-gate`, `valence-evidence-index`, `valence-evidence-release`, and `valence-object-evidence` continue to pass.
- Cairn validation and proposal/design/tasks gates pass.

## Non-claims

This slice proves only that selected Function Object v1 DTO/hash/caveat construction is owned by the in-repo Valence core crate. It does not prove standalone repository readiness, published package consumability, behavioral correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.
