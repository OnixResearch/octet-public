# Typed contracts for the reviewed Vermilion-to-Lean provider profile and pilot.
#
# Vermilion keeps Verus's Rust front/middle end and replaces the SMT backend
# with explicit Lean theorem obligations. These contracts pin provider inputs,
# the common bounded-increment semantics, and the comparison vocabulary. They
# do not assert that any provider ran.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let TrueOnly = std.contract.from_predicate (fun value => value == true) in
let Width64 = std.contract.from_predicate (fun value =>
  std.is_number value && value == 64
) in
let FalseOnly = std.contract.from_predicate (fun value => value == false) in
let NonClaimArray = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.all (fun item => std.is_string item && std.string.length item > 0) value
  && std.array.elem "does not prove Vermilion, Verus, or Lean soundness" value
  && std.array.elem "does not prove whole-program correctness" value
  && std.array.elem "does not prove semantic equivalence" value
) in
let ExecutionLimits = {
  timeout_ms | PositiveNumber,
  stdout_max_bytes | PositiveNumber,
  stderr_max_bytes | PositiveNumber,
} in
let PinnedDependency = {
  name | NonEmptyString,
  revision | GitRevision,
} in
{
  VerusForkProfile = {
    repository | OneOf ["https://github.com/ilyasergey/verus"],
    branch | OneOf ["dev"],
    revision | OneOf ["48600836424a79420589c56d86e720236a029811"],
    vstd_revision | OneOf ["48600836424a79420589c56d86e720236a029811"],
  },
  TranslatorProfile = {
    name | OneOf ["vermilion_direct+vrml_gen"],
    revision | GitRevision,
    argv | NonEmptyArray | Array NonEmptyString,
  },
  LeanProfile = {
    toolchain | OneOf ["leanprover/lean4:v4.33.0"],
    mathlib_revision | GitRevision,
    lake_manifest_blake3 | Blake3,
    packages | NonEmptyArray | Array PinnedDependency,
  },
  CheckerProfile = {
    checker_id | OneOf ["vrml_check"],
    revision | GitRevision,
    argv | NonEmptyArray | Array NonEmptyString,
    requires_twin_check | TrueOnly,
  },
  AssumptionPolicy = {
    allowed_axioms | NonEmptyArray | Array NonEmptyString,
    proof_holes | NonEmptyArray | Array NonEmptyString,
    requires_complete_declaration | TrueOnly,
    require_twin_check | TrueOnly,
  },
  VermilionLeanProviderProfile = {
    schema_version | OneOf ["octet-vermilion-lean-provider-profile/v1"],
    profile_id | NonEmptyString,
    provider_id | OneOf ["vermilion-lean"],
    claim_scope | OneOf ["per-obligation-lean-kernel-theorem-over-pinned-verus-frontend"],
    vermilion_revision | OneOf ["7c83bbca96940538eb282a67ca923893e5aa0e06"],
    vermilion_repository | OneOf ["https://github.com/ilyasergey/vermilion"],
    verus_fork | VerusForkProfile,
    translator | TranslatorProfile,
    lean | LeanProfile,
    checker | CheckerProfile,
    environment_allowlist | NonEmptyArray | Array NonEmptyString,
    limits | ExecutionLimits,
    assumption_policy | AssumptionPolicy,
    non_claims | NonClaimArray,
  },
  CompatibilityCanary = {
    canary_id | NonEmptyString,
    category | NonEmptyString,
    execution | OneOf ["pure", "live"],
    expected | OneOf ["pass", "reject", "blocked"],
    test_function | NonEmptyString,
    rationale | NonEmptyString,
  },
  VermilionCompatibilityMatrix = {
    schema_version | OneOf ["octet-vermilion-lean-compatibility-matrix/v1"],
    matrix_id | NonEmptyString,
    provider_id | OneOf ["vermilion-lean"],
    reviewed_profile | NonEmptyString,
    upgrade_dimensions | NonEmptyArray | Array NonEmptyString,
    canaries | NonEmptyArray | Array CompatibilityCanary,
  },
  RequiredObligation = {
    obligation_id | NonEmptyString,
    kind | OneOf ["logical-vc", "overflow", "refinement", "typeclass-evidence"],
  },
  RouteObligations = {
    backend | OneOf ["verus", "vermilion-lean", "aeneas-lean"],
    claim_scope | NonEmptyString,
    required_obligations | NonEmptyArray | Array RequiredObligation,
    refinement_bridge_id | NonEmptyString,
    refinement_bridge | NonEmptyString,
    modeled_library_contracts | Array NonEmptyString,
  },
  PilotSemanticContract = {
    schema_version | OneOf ["octet-proof-semantic-contract/v1"],
    contract_id | OneOf ["octet.bounded_increment.checked/v1"],
    revision | NonEmptyString,
    subject_path | NonEmptyString,
    subject_blake3 | Blake3,
    statement | NonEmptyString,
    statement_blake3 | Blake3,
    width_bits | Width64,
    overflow_behavior | OneOf ["checked"],
    precondition | OneOf ["value < u64::MAX"],
    postcondition | OneOf ["result == Some(value + 1)"],
    result_type | OneOf ["Option<u64>"],
    observable_failures | NonEmptyArray | Array NonEmptyString,
    termination_scope | OneOf ["single-call, no loops"],
    routes | NonEmptyArray | Array RouteObligations,
    non_claims | NonClaimArray,
  },
  NegativeControl = {
    case_id | NonEmptyString,
    mutation | NonEmptyString,
    expected_outcome | OneOf ["failed", "blocked"],
    expected_diagnostic | NonEmptyString,
  },
  VermilionNegativeMatrix = {
    schema_version | OneOf ["octet-vermilion-negative-matrix/v1"],
    cases | NonEmptyArray | Array NegativeControl,
  },
}
