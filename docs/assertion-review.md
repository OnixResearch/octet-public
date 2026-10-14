# Assertion review guide

Tiger Style treats assertions as executable design notes. Review them as part of the contract, not as noise around the contract.

## Assertion targets: arguments, returns, invariants

Assert what the code assumes on entry, what it promises on exit, and what must stay true between those points. Review each function, state transition, or data boundary by naming the contract explicitly instead of counting assertions in the abstract.

- Reviewer prompt: Where does this code assert argument validity, return-value shape, and internal invariants?
- Reviewer prompt: If an assumption is important enough to mention in prose, should it be checked in code instead?
- Checklist: Prefer assertions that pin the contract at the boundary where the assumption matters.

## Pair assertions across boundaries

A good assertion often needs a partner. If a caller asserts a precondition, the callee or downstream boundary should still defend its own assumptions. If a writer encodes a value, the reader or decoder should assert the matching shape on the other side.

- Reviewer prompt: For each caller/callee, write/read, or encode/decode boundary, where is the paired check on the far side?
- Reviewer prompt: If one side changes first, will the other side fail loudly instead of drifting silently?
- Checklist: Look for pairs across API boundaries, persistence boundaries, and serialization boundaries.

## Positive and negative space

Review both what must succeed and what must fail. Positive space covers valid ranges, expected shapes, and allowed transitions. Negative space covers forbidden values, impossible states, and malformed inputs that should trip the code immediately.

- Reviewer prompt: Which valid cases prove the happy path, and which invalid cases prove the boundary is real?
- Reviewer prompt: What dangerous nearby input should fail here but currently slips through?
- Checklist: Do not stop at representative valid inputs; name the invalid edge that should be rejected.

## Implication assertions and split checks

When the design says "if a, then b," keep that implication readable in code. Split compound checks when one large assertion hides which subcondition failed or mixes unrelated facts into one opaque predicate.

- Reviewer prompt: Does any implication in the design show up as an explicit check such as `if a { assert!(b) }`?
- Reviewer prompt: Would two smaller assertions make failures easier to localize than one compound assertion?
- Checklist: Split compound assertions when the combined predicate hides which contract clause broke.

## Compile-time assertions

Some relations belong at compile time: layout facts, constant bounds, feature combinations, and other design-time truths that should never wait for runtime traffic. Prefer compile-time assertions when the design can be checked before the program runs.

- Reviewer prompt: Which invariants here are constant relations that could move to a compile-time assertion?
- Reviewer prompt: Are we paying runtime cost for a property that the build could reject earlier?
- Checklist: Move static facts to compile-time checks when possible so runtime assertions stay focused on live state.

## Invalid-path testing

Assertions are stronger when tests drive the code through invalid paths on purpose. Review whether the test plan covers malformed inputs, forbidden transitions, failed preconditions, and other cases that should trip the assertions instead of relying on luck.

- Reviewer prompt: Which invalid-path tests prove the assertions fire on bad inputs or illegal state transitions?
- Reviewer prompt: Which edge case is easy to name in review but still missing from the test suite?
- Checklist: Add tests for boundary violations, malformed data, and impossible-state attempts, not only success cases.

## Reviewer prompts and failure handling

This checklist matters because assertions make design intent reviewable. They tell future maintainers which assumptions are load-bearing, where boundaries are defended, and which failures are programmer errors that should fail fast instead of being papered over.

Tiger Style assertion review is not only about raw density. One precise assertion in the right place can matter more than many generic checks. The goal is to make assumptions explicit, pair checks across boundaries, and back them with invalid-path tests that prove the contract bites when it should.

- Reviewer prompt: If this assertion failed in production, would it expose a programmer error immediately instead of turning it into a vague downstream symptom?
- Reviewer prompt: Does the review note explain why each important assertion exists, not only that an assertion exists?
- Checklist: Send work back when the code hides programmer errors, lacks paired checks, or omits invalid-path coverage for important boundaries.
