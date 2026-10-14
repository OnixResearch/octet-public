# First checked Rust patch

This walkthrough starts from one ordinary Rust patch, runs the existing Octet
static-lint path, fixes the finding, and verifies the saved local evidence. It is
intended for a repository that already follows the setup in
[`docs/consumer-guide.md`](../../consumer-guide.md); production onboarding details
stay there.

## Before patch

The patch adds a pure helper that computes average bytes per completed operation:

```rust
pub fn average_bytes(total_bytes: u64, completed_operations: u64) -> u64 {
    total_bytes / completed_operations
}
```

The code is small, deterministic, and ordinary Rust. The issue is that
`completed_operations` can be zero, so the division has no explicit guard.

## Run Octet and inspect the finding

Run Octet with a local artifact directory:

```bash
cargo octet check --artifact-dir artifacts/octet --output-format sarif
```

For this patch, the selected surface is the existing `unchecked_division` lint.
The deterministic finding shape is a division whose divisor lacks an explicit
zero guard tied to the same value. The diagnostic should name the lint and point
at the division expression; the exact human wording can evolve with the lint, but
the review reason remains the missing explicit zero guard.

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

The bounded remediation rationale is local and reviewable: the fixed code names
the zero-operation case, rejects it before division, and preserves a pure
function that is easy to test without standing up the world.

## Save and verify local evidence

Rerun the same artifact-producing check after applying the fix:

```bash
cargo octet check --artifact-dir artifacts/octet --output-format sarif
```

Expected local outputs include:

- `artifacts/octet/command.txt` with the replayable command;
- `artifacts/octet/status.json` with exit status, counts, selected profile, and
  replay metadata;
- `artifacts/octet/summary.txt` with the human effect summary;
- `artifacts/octet/results.sarif` or `artifacts/octet/results.jsonl` with
  findings sidecars when that output is selected;
- `artifacts/octet/provenance.jsonl` with checker identity, evidence tier,
  artifact locator, BLAKE3 identity, and checked scope.

Then verify the artifact bundle before treating it as review evidence:

```bash
cargo octet artifact verify --artifact-dir artifacts/octet
```

Artifact verification recomputes the saved sidecar identities and fails closed on
missing, stale, malformed, or inconsistent evidence.

## Claim boundaries

This walkthrough demonstrates static lint evidence and artifact verification for
the selected scope. It is not a solver model, not a solver counterexample, not a
formal counterexample, not mathematical proof, not whole-program correctness, not
certification, not C performance parity, not a verified runtime, and not a Rust
replacement. Octet does not introduce `requires` or `ensures` syntax for this
path.

For production setup, rollout profiles, and CI storage decisions, continue with
[`docs/consumer-guide.md`](../../consumer-guide.md).
