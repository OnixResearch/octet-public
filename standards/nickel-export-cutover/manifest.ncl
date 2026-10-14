let NonEmptyString = String in
let StandaloneSource = {
  coordinate | NonEmptyString,
  cargo_git_url | NonEmptyString,
  crate_target | NonEmptyString,
  revision | NonEmptyString,
  cargo_source | NonEmptyString,
  flake_source | NonEmptyString,
  release_profile | NonEmptyString,
  local_override_policy | NonEmptyString,
} in
let PackageRow = {
  name | NonEmptyString,
  role | NonEmptyString,
  source_path | NonEmptyString,
  source_b3 | NonEmptyString,
  selected_by_octet | Bool,
} in
let CompatibilityFacade = {
  path | NonEmptyString,
  legacy_manifest_schema | NonEmptyString,
  legacy_spec_schema | NonEmptyString,
  canonical_owner | NonEmptyString,
  disposition | NonEmptyString,
} in
let CutoverManifest = {
  schema_version | NonEmptyString,
  standalone_source | StandaloneSource,
  packages | Array PackageRow,
  compatibility_facade | CompatibilityFacade,
  supported_claim | NonEmptyString,
  non_claims | Array String,
} in
let Revision = "257fafc1c746f1faf156207043a4c826bfb16d49" in
let NonClaim = "Nickel export cutover proves exact source selection and checked legacy projection compatibility only; it does not prove evaluator correctness, arbitrary historical equivalence, policy correctness, secret absence, deployability, build success, or release eligibility" in
{
  schema_version = "octet-nickel-export-cutover-manifest/v1",
  standalone_source = {
    coordinate = "github:OnixResearch/nickel-export",
    cargo_git_url = "https://github.com/OnixResearch/nickel-export",
    crate_target = "nickel-export-core",
    revision = Revision,
    cargo_source = "git+https://github.com/OnixResearch/nickel-export?rev=%{Revision}#%{Revision}",
    flake_source = "github:OnixResearch/nickel-export/%{Revision}",
    release_profile = "cairn/specs/deterministic-nickel-export/spec.md",
    local_override_policy = "local path overrides are diagnostic-only and MUST NOT replace the exact release dependency, flake input, or lock identity",
  },
  packages = [
    {
      name = "nickel-export-core",
      role = "canonical evaluator-neutral functional core",
      source_path = "crates/nickel-export-core/src/lib.rs",
      source_b3 = "b3:77b76fcb72b741c9489c0fe57728e7f563e7baa2345f6b1bf6ee71da32b2a554",
      selected_by_octet = true,
    },
    {
      name = "nickel-export",
      role = "external evaluator and filesystem shell retained outside ordinary Octet runtime checks",
      source_path = "crates/nickel-export/src/lib.rs",
      source_b3 = "b3:da764a6ef5384c1bdfe9fc263f503d142d39b208bbf0d3813b3be491e42313a8",
      selected_by_octet = false,
    },
  ],
  compatibility_facade = {
    path = "crates/octet-standards/src/nickel_export.rs",
    legacy_manifest_schema = "octet-nickel-export-manifest/v1",
    legacy_spec_schema = "octet-nickel-export-spec/v1",
    canonical_owner = "nickel-export-core",
    disposition = "delegating legacy projection only",
  },
  supported_claim = "Octet delegates evaluator-neutral export identity and admission to one pinned standalone core while retaining product policy and a checked legacy projection",
  non_claims = [NonClaim],
} | CutoverManifest
