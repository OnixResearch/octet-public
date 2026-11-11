## Implementation

- [x] I1 Add Rust-owned acceptance receipt DTOs for standards artifact export.
- [x] I2 Extend `export-artifacts` to consume exported profile JSON and emit a structured receipt.
- [x] I3 Add a live CLI regression that verifies receipt schema, config digest, artifact digests, and no raw log fields.

## Verification

- [x] V1 Save focused Rust/Nickel/OpenSpec verification evidence for the emitted receipt path. evidence=`openspec/changes/archive/2026-05-06-emit-runtime-acceptance-receipts/evidence/live-receipt-output.txt`, `openspec/changes/archive/2026-05-06-emit-runtime-acceptance-receipts/evidence/focused-checks.txt`
- [x] V2 Archive after all implementation and verification tasks are complete.
