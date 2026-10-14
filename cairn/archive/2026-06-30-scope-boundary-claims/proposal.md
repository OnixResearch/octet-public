## Why

external evidence tooling earns trust by stating what it does not claim: a correct implementation of a wrong specification is still wrong, and validation is not proof. Octet needs the same explicit boundary. Its value is strong, but it is not whole-program verification, business-requirement validation, or a guarantee that a reviewed suppression is correct.

A dedicated scope-boundary section will make Octet safer to adopt in governance-heavy environments and prevent product positioning from drifting into unsupported claims.

## What Changes

- Add a "What Octet does not claim" section to core docs.
- State that Octet does not prove whole-program correctness, does not validate business/domain requirements, and does not make suppressions correct by their presence.
- State what Octet does provide: static rejection of known-dangerous source shapes, policy checks, reviewable evidence, and explicit manual-review boundaries.
- Add validation that forbidden unqualified correctness claims fail documentation checks.

## Impact

- **Files**: `README.md`, `docs/safety-critical.md`, `docs/consumer-guide.md`, and release/operator docs.
- **Testing**: positive documentation check for required boundary claims; negative check for unqualified correctness/proof/safety guarantees.
