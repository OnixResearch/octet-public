# Strict Rust profiles

Octet strict profiles are opt-in, versioned policy selections for ordinary Rust. A selected profile activates a reviewed deny-level lint floor, raises function-address checking to a declared minimum, and binds the profile into the existing BLAKE3 run evidence.

A passing profile means that the configured Octet source-shape checks passed for the recorded scope and toolchain. It is not a new Rust language, a transitive effect system, a resolved call-graph proof, formal verification, dependency-closure certification, or release eligibility.

## Profiles

| Profile | Intended use | Required core scopes | Function-address floor | Default address scope |
| --- | --- | --- | --- | --- |
| `robust` | Explicit failure, checked conversion/arithmetic, unsafe public API review, TODO rejection, and suppression governance | no | `observe` | workspace |
| `bounded-systems` | `robust` plus bounded control flow, allocation, collections, channels, task lifecycle, lock ordering, and platform-width boundaries | no | `warn` | workspace |
| `deterministic-core` | `robust` plus ambient-input, platform-effect, state, observability, spawning, allocation, and deterministic-output checks in declared cores | yes | `deny` | `core_source_scopes` |

The profiles form a lattice rather than one universal safety score. `bounded-systems` and `deterministic-core` both extend `robust`, but they make different claims about different source scopes.

The Nickel source of truth is [`standards/strict-profiles/profiles.ncl`](../standards/strict-profiles/profiles.ncl). Runtime Rust consumes the checked-in generated JSON. The catalog is validated against Octet's lint metadata so an unknown required lint fails closed.

## Run a profile

```bash
cargo octet check --profile robust --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

A selected profile is authoritative for its required lint set. If consumer `dylint.toml` disables a required lint or assigns it `allow` or `warn`, Octet strengthens the effective level to `deny`. Unrelated thresholds, allowlists, disabled lints, and levels remain consumer-owned.

Source-level Rust suppressions remain possible. Profiles include Octet's suppression-governance lints, so an escape hatch remains an explicit source diff with the existing justification requirements; profiles do not add a silent assume mechanism.

The Nix-packaged runner includes Cargo, the pinned Rust toolchain, Dylint driver/library, and linker closure. A dependency-bearing consumer with build scripts or proc macros can therefore run the package directly; it does not need an ambient consumer dev shell merely to link dependencies.

Within declared core scopes, `concrete_construction_in_core` classifies direct `Type::new()` and `Type::default()` calls. Local-domain and `core`/`alloc`/`std` result types are not external dependency construction, and one direct external call emits at most one finding even when nested in a larger expression. This remains a bounded HIR/source-shape heuristic: external factories with other names and semantic dependency behavior are outside its claim.

## Bootstrap deterministic core

Declare the core paths in canonical consumer configuration:

```toml
[octet]
core_source_scopes = ["crates/domain-core/src/"]
```

`deterministic-core` rejects a missing, empty, absolute, or workspace-escaping scope before Cargo runs. When no explicit `--function-address-path` is supplied, these paths also define its function-address universe.

First produce and review a source-shape baseline without requesting strict admission:

```bash
cargo octet check \
  --function-addresses observe \
  --function-address-path crates/domain-core/src/ \
  --artifact-dir artifacts/octet-observe
```

Then use the reviewed report as the deny-mode baseline:

```bash
cargo octet check \
  --profile deterministic-core \
  --function-address-baseline artifacts/octet-observe/function-addresses.json \
  --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

An explicit operator mode stronger than a profile floor is preserved. Explicit function-address paths and baselines are also preserved; a profile only supplies paths when none were requested.

Function-address evidence remains source-scanner identity. It does not establish HIR/MIR equivalence, semantic dependency closure, compiler correctness, whole-program safety, verifier soundness, or release eligibility.

## Evidence binding

Strict-profile selection is visible or cryptographically bound in:

- `command.txt` through the normalized `--profile` invocation;
- `summary.txt` through `Profile`, `Config hash`, and `Profile hash`;
- `status.json` through `metadata.profile_name`, `metadata.config_hash`, and `metadata.profile_hash`;
- `provenance.jsonl` through `checked_scope = "profile:<PROFILE>"` and artifact BLAKE3 identities;
- `function-addresses.json` when the profile floor enables that rail.

Changing the selected profile or its generated policy changes the BLAKE3 configuration/profile identity. `cargo octet artifact verify` recomputes artifact hashes and checks profile-scoped provenance coverage, but verification remains evidence-integrity validation rather than behavioral proof.

## Compatibility

Without `--profile`, `cargo octet check` retains the existing `workspace-metadata` profile, consumer configuration behavior, hashes, and function-address default. The runner removes any ambient `OCTET_STRICT_PROFILE` value from an unprofiled child process, so shell state cannot silently opt a check into strict behavior.

## Current boundary and next layers

This profile rail composes existing direct source/HIR checks. Its first Valence downstream pilot also demonstrates that packaging success, lint precision, profile admission, and function-address identity are separate observations; none promotes another into semantic proof.

Octet now provides a separate `cargo octet admit --profile deterministic-core --closure` rail for conservative selected source-token effect propagation under exact function-address-bound call summaries. It emits its own receipt and preserves explicit limits around name resolution, macros, traits, MIR, dependency implementations, and runtime behavior; see [`effect-closure-admission.md`](effect-closure-admission.md). Capability-typed trait contracts, compiler-backed closure versions, dependency implementation evidence, and required proof rails remain future evidence layers rather than implicit profile claims.
