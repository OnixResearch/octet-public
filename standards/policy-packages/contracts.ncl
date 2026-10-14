# Local policy-package customs contracts.
let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let OneOf = fun allowed => std.contract.from_predicate (fun value => std.is_string value && std.array.elem value allowed) in
let PackageStatus = OneOf ["draft", "promotable", "existing", "non-candidate"] in
let PackageKind = OneOf ["policy", "profile", "template", "proof-rail", "generated-contract", "contract"] in
let SchemaVersion = std.contract.from_predicate (fun value => value == "octet-policy-package-manifest/v1") in
let GeneratedArtifactContract = {
  path | NonEmptyString,
  generation_command | NonEmptyString,
  validation_command | NonEmptyString,
} in
let PackageManifestContract = {
  schema_version | SchemaVersion,
  package_id | NonEmptyString,
  kind | PackageKind,
  owner | NonEmptyString,
  status | PackageStatus,
  source_paths | Array NonEmptyString | default = [],
  generated_artifacts | Array GeneratedArtifactContract | default = [],
  registry_family_ids | Array NonEmptyString | default = [],
  generation_commands | Array NonEmptyString | default = [],
  validation_commands | Array NonEmptyString | default = [],
  positive_fixtures | Array NonEmptyString | default = [],
  negative_fixtures | Array NonEmptyString | default = [],
  provenance | NonEmptyString,
  license_notes | NonEmptyString,
  non_claims | Array NonEmptyString | default = [],
  manual_review_actions | Array NonEmptyString | default = [],
  remote_fetches_allowed | Bool | default = false,
  secret_exclusion_rationale | String | default = "",
} in
let PackageRulesContract = std.contract.from_predicate (fun package =>
  (std.array.length package.non_claims > 0)
  && (std.array.length package.manual_review_actions > 0)
  && (
    package.status == "draft"
    || package.status == "non-candidate"
    || (
      std.array.length package.positive_fixtures > 0
      && std.array.length package.negative_fixtures > 0
    )
  )
) in
{
  PackageManifest = PackageManifestContract,
  PackageRules = PackageRulesContract,
  GeneratedArtifact = GeneratedArtifactContract,
}
