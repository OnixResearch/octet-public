let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let SchemaVersion = OneOf ["octet-clippy-restriction-profiles/v1"] in
let LintLevel = OneOf ["warn", "deny"] in
let ClaimScope = OneOf ["clippy-restriction-diagnostics"] in
let RestrictionLint = OneOf [
  "dbg_macro",
  "exit",
  "fn_params_excessive_bools",
  "host_endian_bytes",
  "iter_over_hash_type",
  "mem_forget",
  "multiple_unsafe_ops_per_block",
  "undocumented_unsafe_blocks",
] in
let ProfileContract = {
  profile_id | NonEmptyString,
  summary | NonEmptyString,
  lint_level | LintLevel,
  lints | NonEmptyArray | Array RestrictionLint,
  claim_scope | ClaimScope,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
{
  Profile = ProfileContract,
  Catalog = {
    schema_version | SchemaVersion,
    profiles | NonEmptyArray | Array ProfileContract,
  },
}
