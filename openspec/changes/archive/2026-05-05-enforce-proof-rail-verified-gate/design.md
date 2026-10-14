## Context

`PolicyEntry.proof_rails` already models backend, command, source paths, proof paths, pass signal, and trust boundaries. Nickel validates that verified targets declare at least one rail, but the checker does not yet consume that rail as a failing gate.

## Goals / Non-Goals

**Goals:**
- Make verified-mode policies fail when their declared source/proof artifacts are absent from the checked source set.
- Require a loaded proof receipt to contain the configured pass signal.
- Keep the checker deterministic and artifact-backed: Rust consumes exported JSON/artifacts, not Nickel source.

**Non-Goals:**
- Execute arbitrary proof commands inside `tigerstyle-standards check`.
- Claim semantic correctness from text receipts alone; this slice gates local proof evidence shape and trust-boundary accounting.

## Decisions

### 1. Verify loaded artifact shape, not live command execution

**Choice:** The Rust checker validates declared source paths, proof paths, and pass-signal text in the provided source/artifact tree.

**Rationale:** The standards checker remains pure and deterministic, while Nix/OpenSpec evidence can separately run proof commands and save receipts.

**Alternative:** Run proof commands from policy JSON during checks. Rejected because that mixes policy evaluation with shell execution and makes consumer checks non-hermetic.

### 2. Use the verification family

**Choice:** Proof-rail diagnostics run only for selected `verification` policies.

**Rationale:** Existing consumer helpers select explicit families; proof gates should not leak into embedded, architecture, or idioms checks.

## Deterministic Proof Plan

- Artifact export evidence proves proof-rail fields survive Nickel export and generated artifact packaging.
- Fixture evidence proves compliant source plus pass receipt exits zero and noncompliant/missing pass receipt exits nonzero.
- Focused checks prove Rust tests, clippy, policy validation, Nix fixture checks, and OpenSpec validation all pass.
