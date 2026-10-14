## Design

The rail has a pure compatibility core and a thin CLI shell. The core receives old/new contract documents, declared stability policy, and typed schema metadata. It returns compatibility findings and generated-client eligibility. The shell reads files, invokes existing generators, writes receipts, and renders human or JSON output.

### Decisions

1. **Typed DTOs own the contract.** Dropshot-style source annotation is useful, but Octet must keep Rust-owned DTOs and generated Nickel contracts as the source for receipts and policy surfaces. Handwritten JSON/OpenAPI documents are review inputs, not source of truth.

2. **Generated clients are downstream test fixtures.** Progenitor-style client generation is useful for exercising Octet APIs, but generated clients do not prove service correctness or evidence truth. They prove only that a contract can be consumed according to the selected generator profile.

3. **Schema generation stays explicit.** Typify-style Rust type generation from schema is allowed only for external schema inputs whose trust boundary is named. Internal Octet receipt types should derive outward contracts from Rust instead of regenerating core domain types from JSON.

4. **Compatibility diffing is policy-bound.** Drift/openapi-lint/API-manager ideas become deterministic checks: additive changes may pass by profile, breaking changes require explicit change records, and undocumented removals fail closed.

5. **No hidden middleware model.** Dropshot's explicit typed-context lesson is adopted: any authentication, authorization, policy, or evidence role must be present in typed inputs and outputs rather than ambient request state.

6. **Positive and negative corpora are mandatory.** Positive fixtures cover additive schema evolution, accepted deprecations, generated-client smoke calls, and stable receipt JSON. Negative fixtures cover removed required fields, changed enum meanings, malformed schemas, generator failures, and overclaiming client evidence.

### Validation shape

A first implementation should add focused pure tests for compatibility classification, generator eligibility, and malformed-input diagnostics; fixture tests for generated sample clients; and Cairn validation/gates for this change package.
