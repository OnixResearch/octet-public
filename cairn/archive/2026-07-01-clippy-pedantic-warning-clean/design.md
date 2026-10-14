## Context

The baseline pedantic run failed on mechanical issues: clone assignment, underscore-prefixed binding use, match arm duplication, manual let-else, missing public error docs, redundant `#[must_use]`, one sortable comparator, and one oversized standards helper.

## Decisions

### 1. Enable pedantic at the existing Clippy seam

**Choice:** Change `checks.clippy` from `--all-targets -- -D warnings` to `--all-targets -- -D warnings -W clippy::pedantic`.

**Rationale:** The flake check is already the release-readiness Clippy seam. Keeping pedantic there avoids a second, overlapping Clippy check and makes `nix flake check` inherit the stricter policy.

### 2. Fix mechanical findings directly

**Choice:** Apply the current Clippy suggestions or equivalent small refactors, and use narrow local allows only when a pedantic rule conflicts with existing structural constraints.

**Rationale:** Broad `allow(clippy::pedantic)` would hide new regressions. The baseline issues are small and reviewable.

### 3. Preserve separate warning-clean cargo check

**Choice:** Leave `warning-clean` as a cargo-check warning-deny rail and document pedantic coverage under the Clippy rail.

**Rationale:** Rust compiler warnings and Clippy warnings should fail with different command names so operators can triage quickly.

## Risks / Trade-offs

- Pedantic can produce style churn. Mitigate by fixing current findings and avoiding broad allows.
- `too_many_lines` can force larger refactors than this change needs. Mitigate with a narrow allow only when splitting the helper would obscure generated-output ordering.
- Clippy diagnostics can vary across toolchains. Mitigate by using the pinned Nix devshell and flake check.

## Validation

- Positive: `nix develop -c cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes.
- Positive: `nix build .#checks.x86_64-linux.clippy --no-link -L` passes with the same pedantic profile.
- Regression: touched Rust logic still passes the focused relevant tests and Cairn validation/gates pass.
