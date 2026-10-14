# Effect capability ports

Octet keeps deterministic runner and standards decisions in pure cores. Shells own
filesystem traversal, process spawning, environment capture, working directories,
timeouts, clocks, randomness, logging, metrics, network state, async runtime
state, and output writing. Pure cores receive typed in-memory data and return
plans, diagnostics, decisions, or structured events.

The first explicit seam is the proof command process plan. `cargo-octet` builds a
pure `ProcessCommandPlan` from explicit argv, timeout, and environment allowlist
facts before the shell spawns anything. The shell then enacts that plan, captures
the result, and core helpers classify captured process outcomes. This gives tests
a side-effect-free path for positive pass results and negative missing, malformed,
failed, or timed-out process results.

Filesystem capabilities follow the same shell/core split. `cargo-octet` opens
operator-declared workspace, source, artifact, proof, and cache directories as
narrow typed roots, validates every in-root locator as relative, and reads or
writes artifact bundles, lint caches, source files, and proof receipts through
those roots. Traversal, absolute locator, symlink, and non-regular-file cases
fail closed before the shell reads, writes, verifies, or caches outside the
chosen root.

Capabilities stay narrow. `ProcessCapabilityToken` names only the process effect
family; it is not a broad service object and it does not hide filesystem,
environment, clock, randomness, logging, metrics, network, or async-runtime
access. Concrete values are preferred whenever possible.

This is a reviewable effect boundary, not a Rust effect system, not a whole-core
purity proof, and not a guarantee that every dependency is effect-free. Existing
purity-effect lints and capability manifests remain guardrails; they do not turn
source-shape evidence into a complete semantic proof.
