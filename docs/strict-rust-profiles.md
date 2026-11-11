# Strict Rust profiles

Octet strict profiles are opt-in, versioned policy selections for ordinary Rust. A selected profile activates a reviewed deny-level lint floor, raises function-address checking to a declared minimum, and binds the profile into the existing BLAKE3 run evidence.

A passing profile means that the configured Octet source-shape checks passed for the recorded scope and toolchain. It is not a new Rust language, a transitive effect system, a resolved call-graph proof, formal verification, dependency-closure certification, or release eligibility.

## Profiles

| Profile | Intended use | Required core scopes | Function-address floor | Default address scope |
| --- | --- | --- | --- | --- |
| `robust` | Explicit failure, checked conversion/arithmetic, unsafe public API review, TODO rejection, and suppression governance | no | `observe` | workspace |
| `bounded-systems` | `robust` plus bounded resources, platform-width boundaries, and exact nominal-domain policy | no | `warn` | workspace |
| `deterministic-core` | `robust` plus declared core effects, deterministic output, and exact nominal-domain policy | yes | `deny` | `core_source_scopes` |

The profiles form a lattice rather than one universal safety score. `bounded-systems` and `deterministic-core` both extend `robust`, but they make different claims about different source scopes.

The Nickel source of truth is [`standards/strict-profiles/profiles.ncl`](../standards/strict-profiles/profiles.ncl). Runtime Rust consumes the checked-in generated JSON. The catalog rejects an unknown required lint.

The shared Nickel `boolean_api_policy` defines covered direct boolean
parameters, the private minimum, narrow expectations, and prohibited broad
suppression. All three strict profiles require `raw_bool_parameter`; its
exception-governance companion `raw_bool_parameter_suppression` is rejected as
a workspace-wide suppression and becomes a hard error under an explicit deny.
A selected profile binds the policy into its identity rather than importing
consumer warning budgets. See [`boolean-api-modeling.md`](boolean-api-modeling.md).

`bounded-systems` and `deterministic-core` deny exact nominal-domain findings. They never deny suffix-based discovery candidates. The selected profile sets nominal-domain mode to `deny`, also over a consumer `nominal_domain_mode = "inventory"`, so the inventory mode's warning about capping `lint_levels` denies at `warn` is not printed under these profiles.

## Run a profile

```bash
cargo octet check --profile robust --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

A selected profile is authoritative for its required lint set. If consumer `dylint.toml` disables a required lint or assigns it `allow` or `warn`, Octet strengthens the effective level to `deny`. The profile denies are applied after command-line lint options, so `-A` or `-W` flags in `DYLINT_RUSTFLAGS` cannot weaken them either. rustc would still let `--force-warn` or a `--cap-lints` below `deny` override a later deny, so a profiled compilation fails with an error that names the flag when `--cap-lints allow|warn|force-warn` is set or when a `--force-warn` names a required lint directly, through its bare alias, or through a lint group that contains it. Unrelated thresholds, allowlists, disabled lints, and levels remain consumer-owned; explicit command-line levels win over consumer `lint_levels`.

Source-level Rust suppressions remain possible. Profiles include Octet's suppression-governance lints, so an escape hatch remains an explicit source diff with the existing justification requirements: `unjustified_allow` denies an `allow` or `expect` of a Clippy lint, an Octet lint or group (`tigerstyle::...`), or an Octet bare alias such as `no_unwrap` unless a non-doc `//` comment on or directly above the attribute, or a non-empty `reason = "..."`, explains it, and `unjustified_no_todo_allow` audits `no_todo` allows. Under a profile a source `warn` that reaches one of the profile's required lints, such as `#![warn(no_unwrap)]`, `#[warn(tigerstyle::no_unwrap)]`, or a `tigerstyle::` group with a required member, lowers that lint from `deny` and needs the same justification. A `warn` that only raises lints the profile does not require, such as `#![warn(function_length)]`, stays unaffected, and without a profile `warn` is never governed. rustc has no `force_warn` source attribute, so `--force-warn` is handled as a command-line weakening above. Profiles do not add a silent assume mechanism.

The governance lints themselves cannot be suppressed under a profile. rustc applies an attribute's lint levels before a lint inspects that attribute, so `#![allow(tigerstyle::structure)]` or `#[allow(unjustified_allow)]` would otherwise switch the audit off for the whole subtree. With a profile selected, Octet therefore rejects with a compile error any source `allow`, `expect`, or `warn` that you write naming `unjustified_allow` or `unjustified_no_todo_allow`, by bare name, by `tigerstyle::` name, or through the Octet group that contains it (`tigerstyle::structure`, `tigerstyle::safety`). A justification comment or `reason` does not make it legal, so such an attribute gets only this error and no `unjustified_allow` finding asking for a justification. The error is a compiler-session error, not a lint, so no lint level can silence it. The governance lints stay at `deny`, and nothing else is forbidden crate-wide: a raised level such as `#![deny(unjustified_allow)]`, a justified allow of an ordinary floor lint such as `no_unwrap`, your own group forbids, and attributes that another crate's macro or derive generates are unaffected.

Both checks look at the whole attribute list of an item, as rustc does: a lint that a later `deny` or `forbid` on the same node raises back is not lowered. `#[warn(tigerstyle::safety)]` followed by `#[deny(tigerstyle::safety)]` is therefore accepted, while `#[warn(tigerstyle::safety)]` followed by only `#[deny(tigerstyle::no_unwrap)]` still lowers `unjustified_no_todo_allow` and is rejected. A `deny` placed before the lowering attribute does not count.

Both checks also read the literal attributes in a `macro_rules!` body where the macro is defined, whether or not the crate invokes it. That includes each attribute a literal `cfg_attr(predicate, ...)` there expands into. A `#[macro_export]` macro whose body holds `#[allow(tigerstyle::structure)]` is therefore rejected in the crate that defines it, and an unjustified `#[allow(tigerstyle::no_unwrap)]` in that body is reported there. In a crate that only invokes the macro, the expansion counts as another crate's macro output, and so does a `macro_rules!` definition that the expansion produces. Attributes that a macro builds from fragments, such as `#[$meta]`, are checked where the macro expands. In a body, a `///` doc comment between attributes keeps them one attribute list, so a later `deny` still raises a lint back. When a `$(...)` repetition follows a lowering attribute, it may expand into such a `deny`, so that attribute is judged where the macro expands. A local macro's attribute is reported once, however often the macro expands.

The Nix-packaged runner includes Cargo, the pinned Rust toolchain, Dylint driver/library, and linker closure. A dependency-bearing consumer with build scripts or proc macros can therefore run the package directly; it does not need an ambient consumer dev shell merely to link dependencies.

Core scopes are workspace-relative path prefixes. Octet trims each entry, strips a leading `./` and a trailing `/`, and matches whole path components from the workspace root: `crates/core` covers `crates/core/src/lib.rs` but not `crates/core-shell/src/lib.rs`, `domain-core/src` does not cover `crates/domain-core/src/lib.rs`, and a scope may name a single file such as `src/lib.rs`. Rustc file names are made relative to rustc's working directory, which Cargo sets to the workspace root, before matching; rustc names a file by the path it reached it through, without resolving symlinks, so a scope always covers files compiled through its own path. The lint library also resolves each scope on disk, following symlinks and `..` the way profile preparation does, and a scope that resolves inside the root also covers that canonical workspace-relative path: an in-root `linked -> src` scope covers `src/lib.rs`, and `crates/x/../core/src` covers `crates/core/src`. A scope that names no existing file or directory or resolves to the workspace root itself matches nothing, and a scope whose target lies outside the root, such as `src/core -> ../../shared/core`, matches only files compiled through its in-root path; the lint run warns about each and says which. A scope that used to match by substring must be rewritten as a path from the workspace root. The same resolution, matching, and warnings apply to `state_machine_core_scopes` and `state_machine_generated_adapter_scopes`.

Within declared core scopes, `concrete_construction_in_core` classifies direct `Type::new()` and `Type::default()` calls by the constructed `Self` type of the called associated function, so fallible constructors returning `Result<T, _>` or `Option<T>` are classified by `T`. Local-domain and `core`/`alloc`/`std` constructed types are not external dependency construction, and one direct external call emits at most one finding even when nested in a larger expression. This remains a bounded HIR/source-shape heuristic: external factories with other names and semantic dependency behavior are outside its claim.

The default-empty `core_value_provider_types` setting classifies an explicitly reviewed external value ADT by its exact compiler definition path, for example `"reviewed_values::Value"`. Import aliases retain that resolved identity; crate-only names, prefixes, suffixes, and unselected siblings from the same dependency do not match. This changes only the construction heuristic, not lint severity or other effect, resource, or architecture checks. The consumer owns review of every matching type in its locked dependency cohort and must renew that review when dependencies change. Definition paths do not distinguish multiple versions of the same crate: a declaration covers every resolved match. The setting neither verifies provider identity nor proves implementation purity.

## Bootstrap deterministic core

Declare the core paths in canonical consumer configuration:

```toml
[octet]
core_source_scopes = ["crates/domain-core/src/"]
```

`deterministic-core` rejects a missing, empty, absolute, or workspace-escaping scope before Cargo runs; a scope escapes when it has a `..` component or when its symlink-resolved path leaves the workspace root, and the error names `core_source_scopes` in `dylint.toml`. When no explicit `--function-address-path` is supplied, these paths also define its function-address universe. Octet reads these scopes, and every other lint setting, only from the workspace `dylint.toml`; `cargo octet check` and `cargo octet admit` exit 2 before Cargo runs if `DYLINT_TOML` is set, because Dylint would otherwise lint under that variable's config while the evidence names the file's.

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

An explicit operator mode stronger than a profile floor is preserved. Explicit function-address paths and baselines are also preserved; a profile only supplies paths when none were requested. `--function-address-path` and `--function-address-baseline` need `--function-addresses observe|warn|deny` or a strict profile (from `--profile` or an architecture admission binding), and each effective function-address path, explicit or profile-supplied, must stay inside the workspace root after resolving symlinks; otherwise the check exits 2 before Cargo runs and names the flag or config entry that supplied the path. While collecting, a symlinked directory or `.rs` file inside an effective scope that resolves outside the workspace root is refused, not followed. Without any effective path, as under `robust` and `bounded-systems`, the whole workspace root is scanned, and there a symlinked directory resolving outside the root, such as a Nix `result` link, a nix-direnv `.direnv` input, or a crate linked in from elsewhere in a monorepo, is skipped without being followed; a symlinked `.rs` file resolving outside the root is still refused. The workspace-relative path of each skipped link is recorded in `function-addresses.json` under `skipped_directory_links`, and a `note:` line on stderr names the path and its target, so a linked-in crate that Cargo lints but the scan left out stays visible. The target is left out of the artifact because it is machine-specific (a home directory, or a Nix store path that changes on every rebuild) and would make the artifact non-reproducible; see [function fingerprints](function-fingerprints.md#source-scanner-boundaries).

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
