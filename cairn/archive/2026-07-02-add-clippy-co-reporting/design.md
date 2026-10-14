## Context

Clippy is the Rust ecosystem's broad hygiene lint suite. Octet should interoperate with it without duplicating Clippy rules or blurring which checker produced which diagnostic.

## Approach

1. **Two modes.** Support run mode, where Octet invokes Clippy, and import mode, where Octet reads saved Clippy JSON.
2. **External-lint DTO.** Define a small normalized external-lint record that preserves source checker, lint name, level, message, span, suggestions, and gate outcome.
3. **Registered checker identity.** Extend provenance producer validation to accept the Clippy checker ID for `static_lint` evidence when the rail is enabled.
4. **Artifact verification.** Make Clippy receipts optional artifacts: verified when present, not required for default Octet bundles.

## Decisions

### Preserve Clippy identity

**Choice:** Clippy findings stay external findings under a Clippy checker ID.

**Rationale:** Users need to know whether remediation belongs to Clippy configuration or Octet lint policy.

### Keep hosted scanners out of scope

**Choice:** Do not include SonarQube, CodeQL, Semgrep, or hosted code-scanning platforms in this slice.

**Rationale:** Local Clippy co-reporting is enough to define the external-lint seam without introducing service dependencies.

## Risks / Trade-offs

- **Message-format churn:** Cargo/Clippy JSON can change. Mitigate with fail-closed parser diagnostics and fixtures.
- **Duplicate diagnostics:** Clippy and Octet may flag similar code. Mitigate by preserving source checker and avoiding deduplication in v1.
- **Runtime cost:** Clippy can be slower than Octet-only linting. Mitigate by opt-in run/import modes.

## Validation

- Import-mode fixture with known Clippy JSON.
- Malformed JSON and missing-field negative tests.
- Run-mode smoke inside the Nix devshell.
- Artifact verifier accepts optional Clippy receipt with provenance.
- Cairn validate and gates pass for this change.
