let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let RootRelativePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !(std.string.is_match "^/" value)
  && !(std.string.is_match "(^|/)\\.\\.(/|$)" value)
) in
let Blake3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let PromotionLane = {
  lane | OneOf ["inventory", "advisory", "warning", "error", "migration"],
  blocking | Bool,
} in
let RuleCoverage = {
  rule_id | NonEmptyString,
  lane | OneOf ["inventory", "advisory", "warning", "error", "migration"],
  positive_cases | Number,
  negative_cases | Number,
} in
{
  Profile = {
    schema_version | OneOf ["octet-ast-grep-structural-profile/v1"],
    profile_id | NonEmptyString,
    tool | {
      name | OneOf ["ast-grep"],
      version | NonEmptyString,
    },
    sgconfig_path | RootRelativePath,
    bundle_inputs | NonEmptyArray | Array RootRelativePath,
    bundle_identity | Blake3Identity,
    scan_scope | {
      paths | NonEmptyArray | Array RootRelativePath,
      globs | Array NonEmptyString,
    },
    severity_posture | OneOf ["inventory", "advisory", "warning", "error"],
    promotion_lanes | NonEmptyArray | Array PromotionLane,
    rule_coverage | NonEmptyArray | Array RuleCoverage,
    codemod_policy | {
      mode | OneOf ["dry-run-only", "interactive"],
      preserve_proposed_diff | Bool,
      validation_commands | NonEmptyArray | Array NonEmptyString,
    },
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
}
