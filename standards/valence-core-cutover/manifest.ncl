let NonEmptyString = String in
let NonClaims = Array String in
let StandaloneSource = {
  coordinate | NonEmptyString,
  cargo_git_url | NonEmptyString,
  crate_target | NonEmptyString,
  revision | NonEmptyString,
  cargo_source | NonEmptyString,
  hardened_profile_archive | NonEmptyString,
  local_override_policy | NonEmptyString,
} in
let ModuleRow = {
  module_or_schema | NonEmptyString,
  standalone_path | NonEmptyString,
  standalone_source_b3 | NonEmptyString,
  legacy_fixture_path | NonEmptyString,
  compatibility_disposition | NonEmptyString,
  fixture_refs | Array String,
  non_claims | NonClaims,
} in
let FixtureRow = {
  fixture_id | NonEmptyString,
  module_or_schema | NonEmptyString,
  expected_behavior | NonEmptyString,
  compatibility_command | NonEmptyString,
  non_claims | NonClaims,
} in
let MigrationTarget = {
  consumer | NonEmptyString,
  current_dependency | NonEmptyString,
  target_dependency | NonEmptyString,
  validation_command | NonEmptyString,
  non_claim | NonEmptyString,
} in
let CutoverManifest = {
  schema_version | NonEmptyString,
  standalone_source | StandaloneSource,
  modules | Array ModuleRow,
  fixtures | Array FixtureRow,
  migration_targets | Array MigrationTarget,
  supported_claim | NonEmptyString,
  non_claims | NonClaims,
} in
let ValenceRevision = "5f1c2ba5072c6f9622fa59b1af20502985f569fd" in
let CutoverNonClaim = "standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility" in
{
  schema_version = "octet-valence-core-cutover-manifest/v2",
  standalone_source = {
    coordinate = "github:OnixResearch/valence?dir=crates/valence-core",
    cargo_git_url = "https://github.com/OnixResearch/valence.git",
    crate_target = "valence-core",
    revision = ValenceRevision,
    cargo_source = "git+https://github.com/OnixResearch/valence.git?rev=%{ValenceRevision}#%{ValenceRevision}",
    hardened_profile_archive = "cairn/archive/2026-07-12-harden-preserves-integrity-boundaries",
    local_override_policy = "local path overrides are development-only diagnostics and MUST NOT replace the exact release dependency or lock identity",
  },
  modules = [
    {
      module_or_schema = "function_object",
      standalone_path = "crates/valence-core/src/function_object.rs",
      standalone_source_b3 = "b3:90165fa493397aa7fbd620d191ebb5990b355b42a71f5310b306f8b0d32c435c",
      legacy_fixture_path = "crates/valence-core/src/function_object.rs",
      compatibility_disposition = "legacy-fixture",
      fixture_refs = ["function-object-v1-compatible"],
      non_claims = [CutoverNonClaim],
    },
    {
      module_or_schema = "evidence_index",
      standalone_path = "crates/valence-core/src/evidence_index.rs",
      standalone_source_b3 = "b3:75320ed218e1a726745ad166b394c2a640159ec08863fa7e231e2f221d780c4e",
      legacy_fixture_path = "crates/valence-core/src/evidence_index.rs",
      compatibility_disposition = "legacy-fixture",
      fixture_refs = ["evidence-index-compatible"],
      non_claims = [CutoverNonClaim],
    },
    {
      module_or_schema = "evidence_codebase_index",
      standalone_path = "crates/valence-core/src/evidence_codebase_index.rs",
      standalone_source_b3 = "b3:20c34a80ed5cfd276617e76315dd58becad2a134d2d312d0a80a300436f200d2",
      legacy_fixture_path = "crates/valence-core/src/evidence_codebase_index.rs",
      compatibility_disposition = "legacy-fixture",
      fixture_refs = ["evidence-codebase-index-compatible"],
      non_claims = [CutoverNonClaim],
    },
  ],
  fixtures = [
    {
      fixture_id = "function-object-v1-compatible",
      module_or_schema = "function_object",
      expected_behavior = "Function Object v1 DTO and object hash behavior remains byte-compatible for checked fixtures",
      compatibility_command = "cargo test -p cargo-octet object -- --nocapture",
      non_claims = [CutoverNonClaim],
    },
    {
      fixture_id = "evidence-index-compatible",
      module_or_schema = "evidence_index",
      expected_behavior = "current-head evidence index render/check behavior remains byte-compatible for checked fixtures",
      compatibility_command = "cargo test -p cargo-octet evidence_index -- --nocapture",
      non_claims = [CutoverNonClaim],
    },
    {
      fixture_id = "evidence-codebase-index-compatible",
      module_or_schema = "evidence_codebase_index",
      expected_behavior = "evidence-codebase-index build/query/html behavior remains byte-compatible for checked fixtures",
      compatibility_command = "cargo test -p cargo-octet evidence_codebase_index -- --nocapture",
      non_claims = [CutoverNonClaim],
    },
  ],
  migration_targets = [
    {
      consumer = "Aspen",
      current_dependency = "git package valence-core from github:OnixResearch/octet",
      target_dependency = "git+https://github.com/OnixResearch/valence.git?rev=%{ValenceRevision}",
      validation_command = "cargo test -p cargo-octet valence_cutover -- --nocapture; cargo test -p cargo-octet evidence_index -- --nocapture; cargo test -p cargo-octet object -- --nocapture",
      non_claim = CutoverNonClaim,
    },
  ],
  supported_claim = "cutover validation proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility",
  non_claims = [CutoverNonClaim],
} | CutoverManifest
