## ADDED Requirements

### Requirement: Slotcar Review Identity Registry [r[slotcar-review-identity-registry]]

The system MUST support a review identity registry that maps mutable human-readable review labels, evidence aliases, policy names, and fingerprint manifest handles to immutable content-addressed targets.

#### Scenario: Label resolves to immutable review target [r[slotcar-review-identity-registry.scenario.label-resolves-target]]

- GIVEN a fingerprint review manifest references a review label or evidence alias
- WHEN registry validation runs
- THEN the label resolves to an immutable target hash and target kind
- AND the manifest output preserves both the human label and immutable target hash

#### Scenario: Pointer history supports rollback [r[slotcar-review-identity-registry.scenario.pointer-rollback]]

- GIVEN a review label is updated to point at a newer evidence target
- WHEN pointer history is inspected or rolled back
- THEN the previous immutable target remains visible
- AND rollback restores the label to the prior target without mutating either target hash

### Requirement: Slotcar Review Identity Diagnostics [r[slotcar-review-identity-diagnostics]]

The system MUST diagnose stale, missing, or mismatched review identity pointers without claiming that identity matches prove semantic correctness.

#### Scenario: Missing identity is diagnosed [r[slotcar-review-identity-diagnostics.scenario.missing-identity]]

- GIVEN a fingerprint review manifest names a registry label that does not exist
- WHEN strict review identity validation runs
- THEN validation fails with the missing label and expected target kind
- AND the diagnostic names the manifest entry that referenced it

#### Scenario: Identity match avoids correctness claim [r[slotcar-review-identity-diagnostics.scenario.no-correctness-claim]]

- GIVEN a review identity pointer resolves and the fingerprint hash matches
- WHEN human or machine output is generated
- THEN the output states that the reviewed identity and source body match
- AND it does not state that the function is correct, safe, or verified solely because the identity matches
