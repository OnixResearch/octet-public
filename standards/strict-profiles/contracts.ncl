let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let SchemaVersion = OneOf ["octet-strict-rust-profiles/v1"] in
let FunctionAddressMode = OneOf ["off", "observe", "warn", "deny"] in
let FunctionAddressScope = OneOf ["workspace", "core"] in
let ClaimScope = OneOf ["direct-source-shape-policy"] in
let EffectFamily = OneOf ["concurrency", "failure", "resource", "time", "unknown"] in
let StrictProfileContract = {
  profile_id | NonEmptyString,
  summary | NonEmptyString,
  required_lints | NonEmptyArray | Array NonEmptyString,
  effect_families | NonEmptyArray | Array EffectFamily,
  requires_core_scopes | Bool,
  minimum_function_address_mode | FunctionAddressMode,
  function_address_scope | FunctionAddressScope,
  claim_scope | ClaimScope,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
{
  StrictProfile = StrictProfileContract,
  Catalog = {
    schema_version | SchemaVersion,
    profiles | NonEmptyArray | Array StrictProfileContract,
  },
}
