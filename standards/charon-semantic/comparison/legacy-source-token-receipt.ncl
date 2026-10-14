let NonEmptyString = std.contract.from_predicate (fun value => std.string.is_match "^.+$" value) in
let B3Identity = std.contract.from_predicate (fun value => std.string.is_match "^b3:[0-9a-f]{64}$" value) in
let Fact = {
  key | NonEmptyString,
  value | NonEmptyString,
} in
let Receipt = {
  schema_version | std.contract.Equal "octet-legacy-source-token-analysis/v1",
  checker_id | std.contract.Equal "octet-standards",
  analysis_kind | std.contract.Equal "selected-source-token-effect-closure",
  status | std.contract.Equal "recorded",
  source_identity | B3Identity,
  selected_item | std.contract.Equal "real_fixture::safe_add",
  facts | Array Fact,
  claims_property_authority | std.contract.Equal false,
  non_claims | Array NonEmptyString,
} in
{
  schema_version = "octet-legacy-source-token-analysis/v1",
  checker_id = "octet-standards",
  analysis_kind = "selected-source-token-effect-closure",
  status = "recorded",
  source_identity = "b3:fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
  selected_item = "real_fixture::safe_add",
  facts = [
    { key = "function.real_fixture::safe_add.present", value = "true" },
    { key = "function.real_fixture::safe_add.unsafe", value = "false" },
    { key = "effect.panic", value = "true" },
    {
      key = "source.identity",
      value = "b3:fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
    },
  ],
  claims_property_authority = false,
  non_claims = [
    "Source-token analysis does not perform complete Rust name resolution, macro expansion, trait dispatch, MIR call-graph analysis, or runtime reachability analysis.",
    "Recorded agreement with a Charon projection does not prove semantic equivalence or effect absence.",
  ],
} | Receipt
