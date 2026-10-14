## Context

external evidence tooling's public site frames its product around a workflow shift: agents generate more code than human review can absorb, so verification must move into the checked substrate. Octet has a different substrate and scope. It stays Rust-compatible and routes code through lints, policy checks, reviewable suppressions, and evidence artifacts.

The design goal is a precise adoption of that framing, not a copy of external evidence tooling's claims. Octet should describe a checked path for systems code and name the evidence surfaces that make the path reviewable.

## Approach

1. Define one canonical sentence and reuse it in the README introduction and consumer guide.
2. Add a compact flow block that maps source Rust to Octet checks, policy, evidence, and CI/release gates.
3. Keep the wording compatible with current behavior: Octet checks source shapes and policy conformance; it does not execute a runtime or prove whole-program correctness.
4. Add a small documentation validation script or transcript command that greps for the canonical phrase and rejects forbidden overclaims.

## Risks

- Overclaiming would reduce trust. Mitigate by making forbidden terms part of the verification task and by linking to the scope-boundary Cairn change.
- Product copy could drift from command behavior. Mitigate by anchoring the flow to existing commands: `cargo octet check`, SARIF output, artifact directories, and release smoke transcripts.
- The phrase could become vague marketing. Mitigate by requiring the flow to name concrete artifacts and gates.

## Validation

- Positive: README and consumer guide contain the canonical sentence and the source-to-gate flow.
- Negative: documentation validation finds no unqualified claim that Octet is a verified runtime or proves program correctness.
- Cairn validation and proposal/design/tasks gates pass for this change.
