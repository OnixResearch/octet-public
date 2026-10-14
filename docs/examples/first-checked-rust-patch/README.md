# First checked Rust patch

This walkthrough starts with an ordinary Rust patch. It runs Octet, fixes one finding, and verifies the saved evidence.

First, complete the setup in [`docs/consumer-guide.md`](../../consumer-guide.md). That guide also defines the production onboarding process.

## Before patch

The patch adds a pure helper that computes average bytes per completed operation:

```rust
pub fn average_bytes(total_bytes: u64, completed_operations: u64) -> u64 {
    total_bytes / completed_operations
}
```

The code has a deterministic finding shape. However, `completed_operations` can be zero. The division has a missing explicit zero guard.

## Run Octet and inspect the finding

Run Octet with a local artifact directory:

```bash
cargo octet check --artifact-dir artifacts/octet --output-format sarif
```

This patch uses the `unchecked_division` lint.

The finding identifies a division without an explicit guard for its divisor. The diagnostic names the lint and points to the division expression.

The exact diagnostic text can change. The review reason remains the missing zero guard.

A representative finding summary is:

```text
unchecked_division: division uses `completed_operations` without a prior zero guard
```

## Fixed patch

Keep the logic in the functional core and return a recoverable error instead of
leaving a panic-shaped edge for callers:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AverageBytesError {
    NoCompletedOperations,
}

pub fn average_bytes(
    total_bytes: u64,
    completed_operations: u64,
) -> Result<u64, AverageBytesError> {
    if completed_operations == 0 {
        return Err(AverageBytesError::NoCompletedOperations);
    }

    Ok(total_bytes / completed_operations)
}
```

The bounded remediation rationale is direct: name the zero-operation case and reject it before division. The function remains pure and easy to test.

## Save and verify local evidence

Rerun the same artifact-producing check after applying the fix:

```bash
cargo octet check --artifact-dir artifacts/octet --output-format sarif
```

Expected local outputs include:

- `artifacts/octet/command.txt` contains the replayable command.
- `artifacts/octet/status.json` contains the exit status, counts, profile, and replay metadata.
- `artifacts/octet/summary.txt` contains the human effect summary.
- `artifacts/octet/results.sarif` or `artifacts/octet/results.jsonl` contains selected finding sidecars.
- `artifacts/octet/provenance.jsonl` contains the checker identity, evidence tier, artifact locator, BLAKE3 identity, and scope.

Then verify the artifact bundle before treating it as review evidence:

```bash
cargo octet artifact verify --artifact-dir artifacts/octet
```

Artifact verification recomputes the saved sidecar identities and fails closed on
missing, stale, malformed, or inconsistent evidence.

## Claim boundaries

This walkthrough demonstrates static lint evidence and artifact verification for the selected scope.

Keep these boundaries:

- The finding is not a solver model.
- The finding is not a solver counterexample.
- The finding is not a formal counterexample.
- The finding is not mathematical proof.
- The result is not whole-program correctness.
- The result is not certification.
- The result is not C performance parity.
- The result is not a verified runtime.
- This path is not a Rust replacement.
- This path does not introduce `requires` or `ensures` syntax.

For production setup, rollout profiles, and CI storage decisions, continue with
[`docs/consumer-guide.md`](../../consumer-guide.md).
