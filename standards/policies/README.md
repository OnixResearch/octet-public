# Tiger Style policy registry

`registry.ncl` maps Tiger Style prose to typed executable policy.

The narrative sources are `docs/standards/source/octet-standards.adoc` and `docs/standards/source/*.adoc`.

The registry gives checkers stable IDs, source anchors, enforcement modes, mechanisms, proof rails, suppression rules, seed manifests, and planned evidence.

The checker uses the existing Rust and Serde JSON stack. It consumes exported registry JSON.

Before you add a custom parser, record why the existing checks are insufficient.

## Files

- `contracts.ncl` — closed Nickel contracts for registry shape, policy entries, severity, checker IDs, enforcement mode taxonomy, mechanism metadata, proof rails, seed manifests, and suppression metadata.
- `registry.ncl` — first-wave policy inventory covering every OpenSpec minimum hard-rule candidate plus advisory and human-process-only guidance, including target-mode escalation metadata.
- `assurance-profiles/` — closed cross-plane profile contracts, the OnixOS lifecycle pilot, positive and negative fixtures, generated JSON, and Nickel Export BLAKE3 identity.
- `minimum-coverage.md` — reviewable mapping from OpenSpec hard-rule candidates to policy IDs and task waves.
- `fixtures/valid/*.ncl` — positive registry contract fixtures.
- `fixtures/invalid/*.ncl` — negative fixtures for missing traceability, unknown modes/checkers, missing suppression metadata, unknown fields, missing structural mechanisms, missing proof rails, target-mode regressions, and proof-relevant seeds without local acceptance.

Policy registry validation and deterministic artifact export:

```bash
nix run .#octet-policy-validate
nix run .#octet-policy-export | jq '.policies | length'
nix run .#octet-policy-artifacts -- /tmp/octet-policy-artifacts
```

The artifact export writes five consumer-facing files:

- `octet-policies.json` — canonical registry JSON consumed by `octet-standards check --policy-artifacts-dir`.
- `octet-check-config.toml` — deterministic checker configuration summary for CI/operator review.
- `octet-seed-manifests.json` — manifest-gated seed corpus metadata.
- `octet-proof-bundles.json` — `octet-proof-bundle/v1` schema for verified proof rails and receipt obligations.
- `octet-config-receipt-families.json` — config/receipt source-of-truth registry for freshness checks.

Assurance profiles remain separate, profile-specific generated artifacts. Build and validate the first pilot with:

```bash
nix build .#octet-assurance-profile-onixos-lifecycle --no-link --print-out-paths
nix build .#checks.x86_64-linux.assurance-profile-onixos-lifecycle --no-link -L
```

The generated manifest binds the profile output with BLAKE3. It does not report evidence success or release readiness.

Rust-emitted evidence DTOs are Rust-derived receipt families.

For each family:

1. Keep the Rust field table as the source of truth.
2. Check in the generated Nickel contract under `standards/evidence/contracts/`.
3. Register the family in `registry.ncl`.
4. Add positive and negative Nickel fixtures.

Check freshness with `cargo test -p cargo-octet evidence_contracts::tests::generated_contracts_are_fresh`.

Check fixture behavior with `cargo test -p cargo-octet evidence_contracts::tests::evidence_contract_fixtures_accept_valid_and_reject_invalid_records`.

Confirm that invalid fixtures fail export:

```bash
for file in standards/policies/fixtures/invalid/*.ncl; do
  ! nix run nixpkgs#nickel -- export --format json "$file" >/dev/null
done
```
