## Implementation

- [x] I1 Add generated-artifact rendering in the standards Rust crate.
- [x] I2 Add `tigerstyle-standards export-artifacts --registry <json> --out-dir <dir>`.
- [x] I3 Add `tigerstyle-standards check --policy-artifacts-dir <dir>`.
- [x] I4 Add Nix app/package plumbing for artifact export.
- [x] I5 Update policy docs with the artifact export path.

## Verification

- [x] V1 Add unit coverage for deterministic artifact content.
- [x] V2 Add CLI coverage proving exported artifacts drive a checker failure.
- [x] V3 Run focused standards tests for artifact export and rendering.
- [x] V4 Run live Nickel-to-artifact export and artifact-backed standards fixture checks.
- [x] V5 Run strict OpenSpec validation and whitespace checks.

## Verification Coverage

| Scenario | Verification |
| --- | --- |
| `r[generated-policy-artifacts.scenario.named-files]` | V2, V4 |
| `r[generated-policy-artifacts.scenario.checker-consumes-artifacts]` | V2, V4 |
| `r[generated-policy-artifacts.scenario.check-config-summary]` | V1, V2, V4 |

## Evidence

- V1/V2/V3: `openspec/changes/archive/2026-05-05-generated-policy-artifacts/evidence/rust-standards.txt`
- V4: `openspec/changes/archive/2026-05-05-generated-policy-artifacts/evidence/policy-artifacts.txt`
- V5: `openspec/changes/archive/2026-05-05-generated-policy-artifacts/evidence/openspec-whitespace.txt`
