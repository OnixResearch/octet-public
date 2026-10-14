let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let Natural = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0
) in
let Positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Blake3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let CohortMember = {
  crate_name | NonEmptyString,
  version | NonEmptyString,
  role | NonEmptyString,
} in
let Severity = OneOf ["advisory", "warning", "error"] in
let Profile = {
  profile_id | NonEmptyString,
  summary | NonEmptyString,
  allowed_artifact_kinds | NonEmptyArray | Array (OneOf ["module", "component"]),
  allowed_proposals | NonEmptyArray | Array NonEmptyString,
  allowed_worlds | Array NonEmptyString,
  allowed_imports | Array NonEmptyString,
  allowed_exports | Array NonEmptyString,
  required_exports | Array NonEmptyString,
  allow_any_exports | Bool,
  resource_limits | {
    max_artifact_bytes | Positive,
    max_imports | Natural,
    max_exports | Natural,
    max_memories | Natural,
    max_tables | Natural,
    max_functions | Natural,
    max_memory_initial_pages | Natural,
    max_memory_maximum_pages | Natural,
    max_table_initial_elements | Natural,
    max_table_maximum_elements | Natural,
    max_function_parameter_words | Natural,
    max_local_words_per_function | Natural,
  },
  allow_memory_grow | Bool,
  allow_start | Bool,
  duplicate_custom_sections | OneOf ["allow", "reject"],
  producer_metadata | OneOf ["ignore", "recorded-only", "required", "reject"],
  severity | {
    malformed | Severity,
    disallowed_kind | Severity,
    disallowed_import | Severity,
    disallowed_world | Severity,
    resource_limit | Severity,
    growth | Severity,
    start | Severity,
    duplicate_metadata | Severity,
    abi_mismatch | Severity,
    external_evidence | Severity,
    cohort_drift | Severity,
  },
  verification_role | OneOf ["recorded_only", "boundary"],
  non_claims | NonEmptyArray | Array NonEmptyString,
  target_runtime | default = null,
} in
{
  Registry = {
    schema_version | OneOf ["octet-wasm-artifact-profiles/v1"],
    cohort | {
      cohort_id | NonEmptyString,
      identity | Blake3Identity,
      members | NonEmptyArray | Array CohortMember,
    },
    profiles | NonEmptyArray | Array Profile,
  },
}
