# Reviewed requirement fixture

This fixture exercises the requirement-contract evaluation surface. It is intentionally narrow.

- The formal-proof clause reuses the checked Aeneas/Lean fixture theorem `aeneas.fixture.safe_add`.
  The retained proof receipt establishes the named theorem over the generated model only; it does
  not establish translation soundness, verifier soundness, whole-program correctness, or release
  eligibility.
- The assumption clause records that the fixture is evaluated against the pinned Lean toolchain
  bytes. The assumption is accepted only with the retained empirical hash-check evidence.
- The excluded clause records that GPU execution paths are outside this fixture's reviewed scope.
