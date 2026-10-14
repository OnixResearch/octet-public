## Context

The config/receipt boundary registry classifies run profiles as Nickel-authored and proof/check receipts as Rust-derived. The missing seam is runtime emission from the standards CLI.

## Decisions

### Rust owns emitted receipt DTOs

**Choice:** Implement the acceptance receipt as Rust `serde` structs.
**Rationale:** The receipt records runtime facts: timestamps, git revision, artifact digests, status, and stage outcomes.
**Rejected:** Hand-authoring emitted receipts in Nickel, because that would make runtime evidence a second source of truth.

### Nickel profile export is the digest boundary

**Choice:** Require `--profile-json` when `--receipt-out` is used and compute `config_digest` from the exact profile JSON bytes consumed by the CLI.
**Rationale:** Operators can reproduce `nickel export --format json standards/profiles/local.ncl` and compare the digest in the receipt.

### No raw logs in acceptance receipts

**Choice:** Include structured warnings/errors and verification commands, not stdout/stderr dumps.
**Rationale:** Raw logs remain debug aids and can contain irrelevant or sensitive detail.
