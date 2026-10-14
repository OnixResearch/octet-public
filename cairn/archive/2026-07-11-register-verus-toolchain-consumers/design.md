## Context

A downstream repository should not infer compatibility solely from matching version strings. Octet's typed profile is the review boundary for which consumers use the exact package and which retain local or semantic-only relationships.

## Goals

- Add explicit reviewed rows for OnixOS lifecycle proofs and Aspen's node-replication probe.
- Reject unknown consumer identifiers at the Nickel contract boundary.
- Keep compatibility distinct from proof validity or runtime admission.

## Non-Goals

- Certify OnixOS's full proof chain or Aspen runtime adoption.
- Change Verus, Rust, solver, or formatter versions.
- Transfer proof receipts between repositories.

## Design

The compatibility contract uses a closed consumer identifier set. The profile adds `onixos-lifecycle` and `aspen-node-replication-pilot` rows with `native` status because both consume the exact Octet profile and production packages. Their rationales bound that native status to package identity only. An invalid fixture supplies an unknown consumer and MUST fail Nickel export. The generated JSON and policy registry remain freshness checked.

## Evidence Boundaries

A native row establishes exact package selection. It does not establish source-model adequacy, proof success, verifier or solver soundness, trusted-boundary discharge, runtime correctness, physical behavior, or release eligibility.
