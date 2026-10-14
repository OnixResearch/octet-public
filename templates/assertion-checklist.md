# Assertion checklist template

## Function, boundary, or invariant under review

- Artifact under review:
- Owner / reviewer:
- Boundary type (function, caller/callee, write/read, encode/decode, invariant):
- Contract summary:
- Notes that must fail fast if violated:

## Assertions on inputs, outputs, and invariants

- Input assumptions to assert:
- Return-value or output properties to assert:
- Invariants that must hold before, during, and after the operation:
- Missing contract check to add now:

## Paired assertion sites

- Boundary 1 and paired checks on both sides:
- Boundary 2 and paired checks on both sides:
- Reader / writer or caller / callee mismatch risk:
- Boundary that still lacks a matching partner check:

## Positive and negative space cases

- Valid cases to cover:
- Invalid or forbidden cases to cover:
- Edge inputs near the boundary:
- Case that should fail loudly but is not yet checked:

## Implications and split assertions

- Design implication to encode (`if a, then b`):
- Existing compound assertion to split:
- Smaller assertions or guards to use instead:
- Failure mode that becomes clearer after splitting:

## Compile-time assertions

- Constant relation or design-time fact to check at compile time:
- Candidate compile-time assertion mechanism:
- Runtime assertion that can stay runtime-only:
- Build-time gap still open:

## Invalid-path tests

- Invalid-path test to add:
- Boundary violation or malformed input to exercise:
- Expected assertion or failure signal:
- Missing negative-space regression test:

## Open risks and follow-ups

- Risk if a listed assertion is still missing:
- Follow-up change or test owner:
- Suppression, exception, or temporary gap to justify:
- Review verdict / next action:
