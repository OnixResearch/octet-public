## Design

Octet receives Trellis proof evidence-chain envelopes as external evidence. The intake core validates identity shape, source envelope linkage, proof IR fields, verifier receipt metadata, assumptions, property IDs, and non-claims. The default output role is recorded-only. Promotion to Octet formal proof evidence requires an Octet-owned rerun receipt with matching inputs and accepted verifier status.

### Decisions

- **Reference intake is default.** Trellis proof IR imported into Octet is recorded-only unless Octet reruns or validates an Octet-owned proof receipt.
- **Promotion is explicit.** Formal-proof role requires Octet proof rail execution and matching source/toolchain/assumption/property identities.
- **Assumption compatibility matters.** Octet rejects promotion when Trellis assumptions are missing, stale, weaker than policy, or outside accepted trust boundaries.
- **Source and IR are both required.** Source envelopes support review/rerun; IR supports typed linkage and policy checks.
- **No downstream semantic overclaim.** Even promoted evidence covers only declared property scope, not whole-program Rust correctness.

### Validation shape

Positive fixtures cover recorded-only Trellis reference intake and an Octet-rerun promotion candidate with matching identities. Negative fixtures cover stale source, missing assumptions, verifier mismatch, property-scope mismatch, command mismatch, role promotion without rerun, and overclaim text.
