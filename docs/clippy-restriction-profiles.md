# Clippy restriction profiles

Octet provides opt-in, Nickel-authored Clippy restriction profiles as compiler-level defense in depth. They complement Octet's semantic source lints and remain a separate `static_lint` evidence rail with checker identity `clippy`.

Do **not** enable `clippy::restriction` as a group. Clippy itself recommends cherry-picking restrictions, and Octet's catalog rejects the blanket group.

## Shipped profiles

| Profile | Posture | Intended scope | Selected restrictions |
| --- | --- | --- | --- |
| `production-safety` | deny | General owned production targets | `dbg_macro`, `mem_forget`, `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block` |
| `canonical-determinism` | deny | Canonical artifacts, evidence codecs, protocol/file-format workspaces | production set plus `host_endian_bytes`, `iter_over_hash_type` |
| `shell-boundary-inventory` | warn | Gradual functional-core/imperative-shell cleanup | `exit` |

The source of truth is [`standards/clippy-restrictions/profiles.ncl`](../standards/clippy-restrictions/profiles.ncl). Runtime Rust consumes the checked-in generated JSON at `standards/clippy-restrictions/generated/profiles.json`. Octet's own flake `clippy` check derives and denies the `production-safety` flags from that generated catalog, so the policy owner dogfoods the general profile without duplicating its lint list.

Octet's strict profiles already cover unwrap, expect-like failure paths, panic/TODO macros, unchecked arithmetic and narrowing, platform-dependent casts, deterministic output flows, and suppression governance with more stack-specific semantics. The Clippy catalog intentionally avoids duplicating those checks.

## Run and verify

Run the semantic Octet profile and Clippy restriction rail as separate, reviewable checks:

```bash
cargo octet check \
  --profile robust \
  --artifact-dir artifacts/octet

cargo octet evidence collect \
  --rail clippy-co-reporting \
  --run \
  --profile production-safety \
  --workspace-root . \
  --artifact-dir artifacts/clippy-restrictions

cargo octet artifact verify \
  --artifact-dir artifacts/clippy-restrictions
```

Profiled run mode uses the embedded shipped catalog by default, so downstream workspaces only need `--profile`. `--config PATH` may select another catalog, which must satisfy the same checked schema and bounded lint allowlist. The runner constructs fixed quiet workspace/all-target Clippy JSON argv and rejects `--command` rather than allowing the profile file or operator to inject arbitrary process authority. Unprofiled Clippy run and JSON import remain compatibility surfaces.

A profiled receipt binds:

- the full catalog BLAKE3 identity;
- the selected profile and its BLAKE3 identity;
- lint level and exact lint IDs;
- exact command argv and workspace scope;
- Clippy findings, source severity, and tool exit code; and
- explicit non-claims.

Denied findings produce rail failure. A nonzero Cargo/Clippy exit without a denied Clippy finding is classified as an integration failure so compilation or tool failure cannot appear clean. Cargo stderr is not parsed as Clippy JSON.

## Scope and exceptions

Use `canonical-determinism` only where output order and byte order are contract surfaces. Native-endian conversions can be correct for kernel or FFI ABIs, and unordered hash iteration can be harmless when no ordered output or replicated decision depends on it. Prefer a narrower profile when the stronger contract does not apply.

When a selected restriction conflicts with a real boundary contract, keep the exception local and reasoned:

```rust
#[allow(
    clippy::host_endian_bytes,
    reason = "Kernel ABI requires native-endian field encoding at this adapter boundary"
)]
fn encode_kernel_field(value: u64) -> [u8; core::mem::size_of::<u64>()] {
    value.to_ne_bytes()
}
```

Re-run positive and negative profile fixtures whenever the pinned Rust/Clippy toolchain changes. A clean profile is bounded compiler-lint evidence; it does not prove behavioral correctness, panic freedom, memory safety, runtime determinism, compiler correctness, formal proof, whole-program correctness, or release eligibility.
