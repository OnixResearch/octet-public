# Closed contracts for Octet's declared durability-boundary policy.
#
# Consumers author a profile in Nickel, export it deterministically, and point
# `durability_profile_path` in `dylint.toml` at the generated JSON. The source
# checks read that generated profile; they never evaluate Nickel themselves.
let NonEmptyString =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.length value > 0
  )
in
let NonEmptyArray =
  std.contract.from_predicate (fun value =>
    std.is_array value && std.array.length value > 0
  )
in
let OneOf = fun allowed =>
  std.contract.from_predicate (fun value =>
    std.is_string value && std.array.elem value allowed
  )
in
let CatalogSchema = OneOf ["octet-durability-claims-catalog/v1"] in
let ProfileSchema = OneOf ["octet-durability-claims-profile/v1"] in
let ObservationClassContract = OneOf [
  "synchronization",
  "adapter-observation",
] in
let SurfaceContract = NonEmptyString in
let BoundaryContract = {
  boundary_id | NonEmptyString,
  source_scope | NonEmptyString,
  boundary_functions | NonEmptyArray | Array NonEmptyString,
  required_observation | ObservationClassContract,
  uncertain_outcome_types | NonEmptyArray | Array NonEmptyString,
  commit_surfaces | Array SurfaceContract | default = [],
  adapter_observations | Array SurfaceContract | default = [],
} in
let has_duplicates = fun values =>
  std.array.length values != std.array.length (std.array.dedup values)
in
let all_non_empty = fun values =>
  std.array.all (fun value => std.string.length value > 0) values
in
let boundary_valid = fun boundary =>
  !has_duplicates boundary.boundary_functions
  && !has_duplicates boundary.uncertain_outcome_types
  && !has_duplicates boundary.commit_surfaces
  && !has_duplicates boundary.adapter_observations
  && all_non_empty boundary.boundary_functions
  && all_non_empty boundary.uncertain_outcome_types
  && (
    boundary.required_observation != "adapter-observation"
    || std.array.length boundary.adapter_observations > 0
  )
in
let ProfileRulesContract = std.contract.from_predicate (fun profile =>
  !has_duplicates (profile.boundaries |> std.array.map (fun boundary => boundary.boundary_id))
  && std.array.all boundary_valid profile.boundaries
) in
let CatalogRulesContract = std.contract.from_predicate (fun catalog =>
  !has_duplicates catalog.observation_classes
  && !has_duplicates catalog.synchronization_surfaces
  && !has_duplicates catalog.flush_surfaces
  && !has_duplicates catalog.recovery_operations
  && !has_duplicates catalog.repair_options
  && all_non_empty catalog.repair_options
  && std.array.all (fun entry => std.string.length entry.finding > 0 && std.string.length entry.repair > 0) catalog.guidance
  && std.array.all (fun finding => std.array.any (fun entry => entry.finding == finding) catalog.guidance) catalog.findings
) in
{
  ObservationClass = ObservationClassContract,
  Boundary = BoundaryContract,
  Profile = {
    schema_version | ProfileSchema,
    profile_id | NonEmptyString,
    boundaries | NonEmptyArray | Array BoundaryContract,
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
  ProfileRules = ProfileRulesContract,
  Catalog = {
    schema_version | CatalogSchema,
    observation_classes | NonEmptyArray | Array ObservationClassContract,
    synchronization_surfaces | NonEmptyArray | Array SurfaceContract,
    flush_surfaces | NonEmptyArray | Array SurfaceContract,
    recovery_operations | NonEmptyArray | Array SurfaceContract,
    findings | NonEmptyArray | Array NonEmptyString,
    repair_options | NonEmptyArray | Array NonEmptyString,
    guidance | NonEmptyArray | Array {
      finding | NonEmptyString,
      repair | NonEmptyString,
    },
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
  CatalogRules = CatalogRulesContract,
}
