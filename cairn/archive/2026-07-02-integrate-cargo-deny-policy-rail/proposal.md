## Why

Octet already classifies dependency advisory, license, duplicate-version, and source-policy checks as a hard DevTools rail, and the policy registry names `cargo-deny` as the checker. That contract is not yet visible as a first-class `cargo octet` evidence rail. Consumers still have to know when to run `cargo deny`, where to store its output, and how to relate the result to Octet provenance.

Octet should adopt `cargo-deny` as the first dependency-audit integration because it covers the deterministic Cargo metadata risks that Octet should not reimplement: advisories, license allow/deny policy, duplicate crate versions, bans, and source allowlists.

## What Changes

- Add a first-class dependency-audit rail backed by `cargo-deny`.
- Add runner support that discovers or accepts a tool-native `deny.toml`, invokes `cargo deny check`, and records the exact argv/config path/workspace scope.
- Emit deterministic dependency-audit summary and JSON receipt artifacts with `policy_check` provenance.
- Add a copyable consumer template or documented config path for controlled rollout.
- Keep legal/license interpretation as a consumer policy decision, not an Octet claim.

## Non-Goals

- Do not replace `cargo-deny`'s policy language or advisory database.
- Do not provide legal advice about license compatibility.
- Do not require dependency-audit execution in the starter `cargo octet check` path until a later rollout explicitly promotes it.
- Do not fetch or update advisory databases implicitly in offline or locked evidence modes.

## Impact

- **Files**: `cargo-octet/src/`, `docs/consumer-guide.md`, `docs/trust-boundary-evidence.md`, `standards/policies/registry.ncl`, templates, tests, and Cairn specs.
- **Behavior**: new local-first dependency-audit command or check option with deterministic receipts.
- **Testing**: positive clean fixture, negative denied dependency fixture, missing config behavior, offline/update-boundary tests, receipt/provenance validation, and Cairn gates.
