# r[impl octet.vermilion_proof_rail.provider_profile]
# Compatibility canaries for the reviewed Vermilion provider cohort.
#
# `pure` canaries are exercised by the octet-standards fixture tests. `live`
# canaries require the pinned Vermilion, Verus fork, Lean, and Mathlib closure
# locally; they are blocked until that closure is built and pinned inside the
# Octet execution environment.
let contracts = import "contracts.ncl" in
{
  schema_version = "octet-vermilion-lean-compatibility-matrix/v1",
  matrix_id = "octet-vermilion-lean-compatibility-7c83bbc-4860083",
  provider_id = "vermilion-lean",
  reviewed_profile = "standards/proof-rails/vermilion-lean/generated/profile.json",
  upgrade_dimensions = [
    "verus_fork",
    "vstd",
    "vermilion",
    "lean",
    "mathlib",
    "lean_smt",
    "cvc5",
    "schema",
    "provider_profile",
  ],
  canaries = [
    {
      canary_id = "pure-complete-receipt",
      category = "positive",
      execution = "pure",
      expected = "pass",
      test_function = "vermilion::tests::complete_contract_fixture_is_supported_but_recorded_only",
      rationale = "a complete synthetic receipt satisfies the typed obligation, axiom, source, and freshness contract",
    },
    {
      canary_id = "pure-source-substitution",
      category = "source-substitution",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "a receipt binding different source bytes cannot inherit the pilot claim",
    },
    {
      canary_id = "pure-spec-substitution",
      category = "spec-substitution",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "a receipt binding another semantic-spec revision or statement cannot be compared",
    },
    {
      canary_id = "pure-weakened-precondition",
      category = "weakened-precondition",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "dropping `value < u64::MAX` or changing overflow behavior changes the claim",
    },
    {
      canary_id = "pure-nat-u64-confusion",
      category = "nat-u64-confusion",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::nat_lemma_route_is_incomparable_with_the_u64_contract",
      rationale = "the existing Verus nat monotonicity lemma is not a u64 checked-increment proof",
    },
    {
      canary_id = "pure-missing-obligation",
      category = "omitted-obligation",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "every required obligation must be accounted for by identity and kind",
    },
    {
      canary_id = "pure-unexpected-axiom",
      category = "unexpected-axiom",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "axioms outside the reviewed allowance, including sorryAx, cannot promote a proof",
    },
    {
      canary_id = "pure-interrupted-check",
      category = "interrupted-checking",
      execution = "pure",
      expected = "reject",
      test_function = "vermilion::tests::negative_controls_fail_at_their_declared_gate",
      rationale = "an interrupted or empty checker result is not success",
    },
    {
      canary_id = "toolchain-absent",
      category = "missing-toolchain",
      execution = "live",
      expected = "blocked",
      test_function = "vermilion::tests::absent_toolchain_is_blocked_not_supported",
      rationale = "Vermilion, Lean, Lake, and cvc5 are absent from the current host",
    },
  ],
} | contracts.VermilionCompatibilityMatrix
