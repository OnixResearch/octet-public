# Tiger Style Policy Registry

`registry.ncl` is the typed index from Tiger Style prose to executable policy. The narrative source remains `docs/standards/source/octet-standards.adoc` and `docs/standards/source/*.adoc`; this registry gives checkers stable IDs, source anchors, current/target enforcement modes, structural mechanisms, proof rails, suppression requirements, seed manifests, and planned evidence.

The first checker foundation uses the existing Rust/Serde JSON stack and consumes exported registry JSON. No new parser dependency is introduced; custom source checks must record why existing rustc, Clippy, rustfmt, Tiger Style, cargo-deny, Tracey, Verus, or flake rails are insufficient before they grow bespoke parsing.

## Files

- `contracts.ncl` — closed Nickel contracts for registry shape, policy entries, severity, checker IDs, enforcement mode taxonomy, mechanism metadata, proof rails, seed manifests, and suppression metadata.
- `registry.ncl` — first-wave policy inventory covering every OpenSpec minimum hard-rule candidate plus advisory and human-process-only guidance, including target-mode escalation metadata.
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

Rust-emitted evidence DTOs are Rust-derived receipt families: keep the Rust field table as the source of truth, check in the generated Nickel contract under `standards/evidence/contracts/`, register it in `registry.ncl`, and include both positive and negative Nickel fixtures. Freshness is enforced by `cargo test -p cargo-octet evidence_contracts::tests::generated_contracts_are_fresh`; fixture behavior is enforced by `cargo test -p cargo-octet evidence_contracts::tests::evidence_contract_fixtures_accept_valid_and_reject_invalid_records`.

Invalid fixtures should fail export:

```bash
for file in standards/policies/fixtures/invalid/*.ncl; do
  ! nix run nixpkgs#nickel -- export --format json "$file" >/dev/null
done
```
