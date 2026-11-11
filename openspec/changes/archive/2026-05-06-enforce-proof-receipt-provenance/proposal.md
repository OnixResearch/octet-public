## Why

Proof receipt hash binding now proves which declared command and source content produced or matches a receipt, but the receipt still lacks a typed provenance contract for the receipt schema, backend identity, runner identity, execution environment, and trust-boundary lines. A passing receipt should be self-describing enough for generated artifact consumers and reviewers to distinguish a current local-command receipt from an unversioned/stale blob with only hashes and a pass signal.

## What Changes

- Add policy-driven proof receipt provenance metadata to verified proof rails.
- Require generated and committed proof receipts to carry deterministic provenance lines when the rail opts in.
- Export provenance obligations through generated policy artifacts.
- Keep default read-only checks and opt-in command execution semantics unchanged.

## Scope

In scope:
- Nickel contracts/registry fields for required proof receipt provenance.
- Rust model/artifact/checker support.
- Fixture, CLI, Nix, docs, and OpenSpec evidence.

Out of scope:
- Networked proof backends.
- Secret, username, timestamp, or ambient environment dumps.
- Full semantic theorem-prover attestation beyond the declared local proof rail.

## Verification

Run focused Rust tests, Nickel policy validation, generated artifact fixture checks, docs checks, OpenSpec validation, and whitespace checks. Save concise transcripts under this change's evidence directory before archive.
