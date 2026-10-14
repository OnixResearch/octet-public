## Context

`nix develop -c bash -c 'cargo check --workspace'` completed, but `cargo-octet` emitted unused-code warnings for the evidence-contract generator and duplicated those warnings across compatibility binaries. The warning source appears to be production module code that is only referenced from tests.

## Decisions

### 1. Classify unused generator code before suppressing warnings

**Choice:** Do not blanket-allow dead code. Either expose the generator through a product command path, move it behind `#[cfg(test)]`, or remove it if it is obsolete.

**Rationale:** Warning suppression would hide whether generated contract freshness is a live operator surface or a test helper.

### 2. Add a warning-deny check after cleanup

**Choice:** Add a focused verification rail that runs the workspace or selected packages with warnings denied under the Nix devshell.

**Rationale:** A warning-clean state is only durable if CI/release checks fail when new warnings appear.

### 3. Cover compatibility binaries

**Choice:** Validate the canonical `cargo-octet` binary and compatibility binaries so alias targets do not carry duplicated warnings.

**Rationale:** Consumers may still invoke compatibility names, and release readiness should not rely on only one binary target being clean.

## Risks / Trade-offs

- Wiring a new command for evidence-contract generation can expand public CLI surface. Mitigate by preferring test-only scope unless operators need the command.
- Denying warnings across all targets may expose dependency or rustc-private churn. Mitigate by starting with owned packages and expanding only after the baseline is stable.
- Some generated-code helpers may intentionally be test-only. Mitigate with clear module-level `#[cfg(test)]` or test support modules instead of broad `allow(dead_code)`.

## Validation

- Positive: the workspace check emits no warnings under the Nix devshell.
- Negative: introducing a dead helper in an owned package fails the warning-deny rail.
- Regression: compatibility binary targets remain warning-clean.
