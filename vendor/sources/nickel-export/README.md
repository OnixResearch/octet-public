# nickel-export

`nickel-export` is the independent, evaluator-neutral boundary for deterministic Nickel export requests, exact-byte identities, diagnostics, receipts, and freshness manifests.

The repository separates a pure core from evaluator and filesystem authority:

- `nickel-export-core` is `#![no_std]` + `alloc`. It normalizes requests, validates complete declared dependency sets, computes BLAKE3 identities, rejects error diagnostics and secret-like material, builds deterministic receipts/manifests, checks freshness, and projects legacy Octet and Mantle shapes. It never evaluates Nickel or performs I/O.
- `nickel-export` is a thin std shell. It reads declared files, invokes an explicit external Nickel program, applies an optional declared contract, writes generated artifacts, and implements fail-closed `--check` mode.

## Claim boundary

An accepted receipt proves exact declared input and output identities under one recorded evaluator descriptor. It does **not** prove evaluator equivalence, deployability, consumer-policy conformance, build success, semantic correctness, or release eligibility.

`declared_only` means the external evaluator did not report its observed import closure. Consumers requiring an evaluator-observed closure must use an adapter that supplies `EvaluatorObservedClosure` evidence to the core. Receipts never conceal this distinction.

## Usage

A request is typed by `onix-nickel-export-request/v1` and names a source, all exact dependencies, import paths, optional selector, optional consumer-owned contract metadata, native output format, and destination. The external CLI interprets non-empty contract metadata as a repository-relative contract file and requires it in `dependencies`; embedded consumers may retain a reviewed contract label while supplying captured diagnostics directly.

```console
nix develop
nickel-export export \
  --spec fixtures/requests/json.json \
  --root . \
  --evaluator nickel \
  --evaluator-identity nixpkgs:nickel \
  --evaluator-version nickel-lang-cli-1.17.0 \
  --manifest fixtures/generated/json.manifest.json \
  --check
```

Use `--write` to update the destination and manifest. Exactly one of `--write` and `--check` is required.

## Schemas and compatibility

Canonical schemas are documented in [docs/schemas.md](docs/schemas.md). Serialization is feature-gated in the core; `--no-default-features` keeps the core evaluator-neutral and `no_std`.

Compatibility projections preserve the checked legacy fields used by:

- Octet `octet-nickel-export-manifest/v1`;
- Mantle `mantle-nickel-export-receipt-v1`.

These are adapters, not alternate semantic owners. Consumer evaluation strategy, destination authority, product policy, and release gates remain outside this repository.

## Validation and release

```console
cargo test --workspace
cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets -- -D warnings
nix flake check -L
```

The typed repository and release profiles live in `config/repository.ncl` and `release/profile.ncl`. Checked JSON exports are freshness-tested. The pinned Nix input, Rust toolchain, Nickel evaluator cohort, AGPL license, positive/negative fixtures, host/Wasm core checks, and CLI tamper tests make the release boundary reproducible.

See [docs/migration.md](docs/migration.md) for the dual-run consumer cutover and rollback procedure.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).

## References

The initial extraction compared these codebases at fixed revisions. They remain references only; they do not transfer consumer-owned policy or evaluator authority into this repository.

- [Octet `nickel_export.rs` at `49d2262d78462c41c7f732eeeda267c78a813606`](https://github.com/OnixResearch/octet/blob/49d2262d78462c41c7f732eeeda267c78a813606/crates/octet-standards/src/nickel_export.rs)
- [Mantle `nickel_export.rs` at `732d0f1a59fb7001d38206321e8576b7c0ec2fda`](https://github.com/OnixResearch/mantle/blob/732d0f1a59fb7001d38206321e8576b7c0ec2fda/src/nickel_export.rs)
- [Cairn policy export shell at `7e9ed636203395b3808a65962f6bb6da60f57268`](https://github.com/OnixResearch/cairn/blob/7e9ed636203395b3808a65962f6bb6da60f57268/crates/cairn-cli/src/policy.rs)
- [Trellis policy checker at `fe008bda65baf9a335fe837294837427973a4ab4`](https://github.com/OnixResearch/trellis/blob/fe008bda65baf9a335fe837294837427973a4ab4/scripts/check-verification-policy.rs)
- [Animus generation checks at `f1a8995dca714938042d66336477aa72c518e0a2`](https://github.com/OnixResearch/animus/blob/f1a8995dca714938042d66336477aa72c518e0a2/flake.nix)
