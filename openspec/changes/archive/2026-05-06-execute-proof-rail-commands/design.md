## Context

The previous verified gate made proof receipts blocking but intentionally stopped short of executing proof commands. The next enforcement step should be explicit because executing commands has side effects and trust-boundary implications.

## Goals / Non-Goals

**Goals:**
- Run only proof commands declared by selected, relevant verified policies.
- Run commands only when the operator opts in with a dedicated CLI flag.
- Treat command output as the proof receipt for the current check run.
- Preserve the existing read-only receipt path when command execution is not requested.

**Non-Goals:**
- Full theorem-prover integration for arbitrary downstream projects.
- Network isolation, VM sandboxing, or remote proof execution.
- Replacing Nickel-authored proof rail metadata with hardcoded Rust policy logic.

## Decisions

### 1. Opt-in proof command execution

**Choice:** Add an explicit CLI flag to `tigerstyle-standards check` that executes relevant proof rail commands before pure checker evaluation.

**Rationale:** Normal generated-artifact checks must remain safe for read-only downstream consumers; command execution is stronger but must be operator-visible.

**Alternative:** Always execute proof commands. Rejected because downstream checks would unexpectedly run arbitrary trusted-registry commands.

### 2. Receipt-as-command-output

**Choice:** The checker injects a synthetic proof receipt containing command, cwd, exit status, stdout, and stderr at the declared proof path for the current run.

**Rationale:** Existing proof-rail diagnostics already validate proof paths and pass signals. Reusing that seam avoids a second policy-evaluation path while making command execution auditable.

**Alternative:** Add a separate command-result report type. Rejected for this slice because the important gate is whether the executed receipt contains the declared pass signal.

### 3. Fixture-local proof commands

**Choice:** The verification fixture policy command runs relative to the checked source root.

**Rationale:** The same policy can be proven against compliant and noncompliant fixture roots without hardcoding repository absolute paths.

## Risks / Trade-offs

**Trusted command boundary** → The flag only executes commands from the selected generated registry/artifact bundle, and docs/evidence must call out that registries are trusted inputs.

**Command portability** → Fixture commands use basic shell and checked-in files so Nix checks can run them deterministically.
