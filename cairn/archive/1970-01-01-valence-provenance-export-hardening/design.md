# Design: Valence provenance export hardening

## Approach

The export path remains Octet-owned. Artifact verification collects deterministic
Octet records, the pure export core normalizes them into adapter-ready values,
and the CLI shell writes the export only after verification and linkage checks
pass.

## Replayable receipt envelope

Each exported record is backed by a receipt envelope containing:

- producer and tool identity;
- tool/checker/policy version identity;
- normalized invocation or command identity;
- input artifact IDs and BLAKE3 digests;
- output artifact IDs and BLAKE3 digests;
- parent receipt or artifact IDs;
- status and failure reason when not passing;
- non-claim boundary text.

The core validates the envelope in memory. The CLI owns reading artifact
directories and writing the final export.

## Typed edge vocabulary

The export uses policy-owned edge kinds such as:

- `checks_source_shape`;
- `records_lint_receipt`;
- `records_policy_receipt`;
- `records_formal_proof_receipt`;
- `imports_trellis_reference`;
- `derives_from_parent_artifact`;
- `feeds_valence_adapter`.

Unknown edge kinds fail closed. Edge semantics describe why records are linked;
they do not promote evidence tiers.

## Tier preservation

Imported Trellis references are exported as manual-review or reference inputs
unless an Octet-owned proof rail reruns the proof and records a passing Octet
receipt. Validation and review artifacts remain validation/review evidence, not
formal proof.

## Fixture strategy

Positive fixtures cover a minimal artifact directory with lint, standards, and
proof receipt records. Negative fixtures cover stale BLAKE3 digest, missing
parent, unknown producer, unsupported edge kind, missing non-claim, and tier
promotion from manual review to formal proof.

## What is explicitly not provided

- No Mantle release bundle inclusion claim.
- No release eligibility or whole-program correctness claim.
- No verifier soundness proof beyond the recorded verifier identity and receipt.
