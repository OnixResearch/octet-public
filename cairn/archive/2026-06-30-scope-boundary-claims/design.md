## Context

Octet already has safety-critical documentation and assertion review guidance, but the highest-level product docs should also state the boundary. This mirrors external evidence tooling's evidence-scope posture while staying accurate to Octet's Rust-compatible lint and policy model.

The section should be short and prominent. It should make both sides visible: unsupported claims are out of scope, and supported evidence is concrete and inspectable.

## Approach

1. Add a scope-boundary section near the README evidence or positioning content.
2. Repeat the boundary in safety-critical or consumer guidance where governance readers land.
3. Use plain wording: Octet does not prove whole-program correctness, validate business requirements, or bless suppressions.
4. Pair every negative statement with a positive supported capability: static lint rejection, policy validation, reviewable suppressions, evidence artifacts, and CI gates.
5. Add documentation checks for required phrases and forbidden overclaims.

## Risks

- Too much caution could make Octet sound weak. Mitigate by pairing boundaries with concrete supported capabilities.
- Docs could diverge between README and guides. Mitigate by using a single canonical bullet list or cross-link.
- Forbidden-phrase checks can be brittle. Mitigate by checking a small set of high-risk unqualified claims instead of broad words like "proof."

## Validation

- Positive: docs contain the required unsupported-claim bullets and supported-capability bullets.
- Negative: docs do not contain unqualified statements that Octet proves correctness, guarantees safety, or validates business requirements.
- Cairn validation and gates pass for this change.
