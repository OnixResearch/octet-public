# Purity And Verus Skill Coverage — Delta Spec

## ADDED Requirements

### Requirement: Full Verus Workflow Coverage In Skill

The Tiger Style skill MUST include the missing Verus workflow guidance
from the reference.

The skill MUST cover:

- when inline `requires`/`ensures` is enough
- when to split into `spec fn` / `proof fn` files
- the two-file or split-directory pattern separating production code and `verus/` specs
- `spec fn`, `exec fn`, `proof fn`, and `#[verifier(external_body)]`
- state-machine invariant proof patterns
- verification command examples
- Verus best practices for invariants, overflow predicates, saturating arithmetic, and time injection

#### Scenario: Complex reusable proof

- GIVEN a proof obligation reused across multiple exec functions
- WHEN an agent follows the skill
- THEN it can identify that the split `verus/` pattern is preferred over crowding the exec file

### Requirement: Functional Core Guidance Includes Verification Context

The skill MUST connect Functional Core / Imperative Shell guidance with
how pure code is tested and verified.

The skill MUST mention that pure core logic is suitable for exhaustive
unit tests, property testing, and formal verification.

#### Scenario: Evaluating a pure helper module

- GIVEN a pure module with no I/O
- WHEN an agent applies the skill
- THEN it recognizes the module as a candidate for property tests and Verus verification guidance
