let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let B3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let RelativePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "^/" value
  && !std.string.is_match "(^|/)\\.\\.(/|$)" value
) in
let FalseOnly = std.contract.from_predicate (fun value => value == false) in
let TrueOnly = std.contract.from_predicate (fun value => value == true) in
let SourceFile = {
  path | RelativePath,
  identity | B3Identity,
} in
let SelectedItem = {
  qualified_name | NonEmptyString,
  kind | OneOf ["function"],
  source_file | RelativePath,
  source_identity | B3Identity,
  begin_line | PositiveNumber,
  end_line | PositiveNumber,
  generated_name | NonEmptyString,
} in
let Theorem = {
  path | RelativePath,
  theorem_id | NonEmptyString,
  declaration_name | NonEmptyString,
  requirement_id | NonEmptyString,
  claim_class | OneOf ["named-theorem-over-generated-model"],
  approved_statement_blake3 | B3Identity,
  source_disposition | OneOf ["maintained"],
} in
let TrustInput = {
  input_id | NonEmptyString,
  identity | B3Identity,
} in
let NonClaims = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.elem "does not prove Aeneas or Charon translation soundness" value
  && std.array.elem "does not prove Lean or verifier soundness" value
  && std.array.elem "does not prove whole-program correctness" value
  && std.array.elem "does not prove semantic equivalence" value
  && std.array.elem "does not establish release eligibility" value
) in
let unique_by = fun select values =>
  std.array.all (fun value =>
    std.array.length (std.array.filter (fun candidate => select candidate == select value) values) == 1
  ) values
in
let ProfileRules = std.contract.from_predicate (fun profile =>
  unique_by (fun source => source.path) profile.source.paths
  && unique_by (fun item => item.qualified_name) profile.source.selected_items
  && unique_by (fun theorem => theorem.theorem_id) profile.theorems
  && unique_by (fun input => input.input_id) (profile.external_models @ profile.trust_inputs)
  && std.array.length profile.outcomes == 3
  && std.array.elem "supported" profile.outcomes
  && std.array.elem "blocked" profile.outcomes
  && std.array.elem "failed" profile.outcomes
  && std.array.all (fun item =>
    std.array.any (fun source => source.path == item.source_file && source.identity == item.source_identity) profile.source.paths
  ) profile.source.selected_items
) in
{
  AeneasConsumerProfile = {
    schema_version | OneOf ["octet-aeneas-consumer-profile/v1"],
    profile_id | NonEmptyString,
    consumer_id | NonEmptyString,
    provider | {
      backend | OneOf ["aeneas-lean"],
      profile_id | OneOf ["octet-aeneas-lean-0.1.0-bc2eb6f"],
      profile_identity | B3Identity,
      compatibility_matrix_id | OneOf ["octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16"],
      compatibility_matrix_identity | B3Identity,
      network_allowed | FalseOnly,
    },
    source | {
      root | RelativePath,
      crate_name | NonEmptyString,
      paths | NonEmptyArray | Array SourceFile,
      selected_items | NonEmptyArray | Array SelectedItem,
      broad_selection_allowed | FalseOnly,
    },
    theorems | NonEmptyArray | Array Theorem,
    external_models | Array TrustInput,
    trust_inputs | Array TrustInput,
    resource_policy | {
      timeout_ms | PositiveNumber,
      stdout_max_bytes | PositiveNumber,
      stderr_max_bytes | PositiveNumber,
      environment_allowlist | NonEmptyArray | Array NonEmptyString,
    },
    outcomes | Array (OneOf ["supported", "blocked", "failed"]),
    trust_closure_complete | TrueOnly,
    non_claims | NonClaims,
  },
  AeneasConsumerProfileRules = ProfileRules,
}
