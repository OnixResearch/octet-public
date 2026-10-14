## Context

Octet's architecture and public API policies are source-shape checks. Release candidates also need a baseline comparison that catches SemVer-breaking public API changes before publication or downstream promotion.

## Approach

1. **Use cargo-semver-checks.** Add a runner shell that invokes `cargo semver-checks` with explicit baseline, package, feature, and target settings.
2. **Baseline as required evidence input.** Pure core validates that a release-evidence run has a baseline identity and selected checked scope before command execution.
3. **Receipt from structured output.** Prefer machine-readable output when available; otherwise preserve bounded logs and normalized finding summaries.
4. **Release-readiness integration.** Make the rail selectable from release-readiness or a dedicated API-stability command, but not from starter lint checks by default.

## Decisions

### Check library API only

**Choice:** Scope v1 to Rust library crates supported by `cargo-semver-checks`.

**Rationale:** That matches the tool's strongest contract and avoids pretending to validate CLI behavior, wire formats, or business semantics.

### Baseline must be explicit

**Choice:** A release-evidence run without baseline fails closed.

**Rationale:** An API diff has no meaning without a named prior API surface.

## Risks / Trade-offs

- **Tool support limits:** `cargo-semver-checks` does not catch every SemVer issue. Mitigate with receipt non-claims.
- **Feature/target drift:** Different feature sets can change the public API. Mitigate by recording exact selected features and target triple.
- **Nightly/rustdoc churn:** The checker depends on rustdoc JSON support. Mitigate by pinning through Nix wrapper inputs where possible.

## Validation

- Positive compatible crate fixture.
- Negative removed-public-item fixture.
- Missing-baseline fail-closed test.
- Receipt tests for feature, target, baseline, and non-claim fields.
- Cairn validate and gates pass for this change.
