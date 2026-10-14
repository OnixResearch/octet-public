let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let Sha256Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let CompatibilityStatus = OneOf ["native", "local-pin", "semantic-reference-only", "unsupported"] in
let CompatibilityConsumer = OneOf [
  "octet",
  "trellis",
  "onixos-lifecycle",
  "aspen-node-replication-pilot",
  "verus-belt",
  "aarch64-linux-formatter",
] in
let FormatterAsset = {
  target | NonEmptyString,
  file_name | NonEmptyString,
  sha256_hex | Sha256Hex,
  nix_sri | NonEmptyString,
} in
let CompatibilityRow = {
  consumer | CompatibilityConsumer,
  verifier_release | NonEmptyString,
  status | CompatibilityStatus,
  rationale | NonEmptyString,
} in
{
  VerusToolchainProfile = {
    schema_version | OneOf ["octet-verus-toolchain-profile/v1"],
    profile_id | NonEmptyString,
    verifier | {
      project | OneOf ["verus-lang/verus"],
      release | NonEmptyString,
      source_revision | GitRevision,
      release_url | NonEmptyString,
      release_sha256_hex | Sha256Hex,
      release_nix_sri | NonEmptyString,
      rust_version | NonEmptyString,
      rust_toolchain_label | NonEmptyString,
      solver_id | NonEmptyString,
    },
    formatter | {
      project | OneOf ["verus-lang/verusfmt"],
      version | NonEmptyString,
      release_tag | NonEmptyString,
      source_revision | GitRevision,
      self_update | OneOf ["disabled-by-wrapper"],
      assets | NonEmptyArray | Array FormatterAsset,
      unsupported_targets | Array NonEmptyString,
    },
    compatibility | NonEmptyArray | Array CompatibilityRow,
    claim_scope | OneOf ["toolchain-and-formatter-identity-inventory"],
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
}
