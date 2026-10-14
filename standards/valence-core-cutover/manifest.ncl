let NonEmptyString = String in
let NonClaims = Array String in
let StandaloneSource = {
  coordinate | NonEmptyString,
  crate_target | NonEmptyString,
  revision_or_local_override | NonEmptyString,
  local_override_policy | NonEmptyString,
} in
let ModuleRow = {
  module_or_schema | NonEmptyString,
  standalone_path | NonEmptyString,
  octet_hosted_path | NonEmptyString,
  compatibility_disposition | NonEmptyString,
  fixture_refs | Array String,
  non_claims | NonClaims,
} in
let FixtureRow = {
  fixture_id | NonEmptyString,
  module_or_schema | NonEmptyString,
  expected_behavior | NonEmptyString,
  compatibility_hash_source | NonEmptyString,
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
{
  schema_version = "octet-valence-core-cutover-manifest/v1",
  standalone_source = {
    coordinate = "github:OnixResearch/valence?dir=crates/valence-core",
    crate_target = "valence-core",
    revision_or_local_override = "pin a reviewed Valence revision for release; local development may use ../valence/crates/valence-core",
    local_override_policy = "local path overrides are development-only and not release evidence unless paired with a reviewed Valence revision and fixture report",
  },
  modules = [
    {
      module_or_schema = "function_object",
      standalone_path = "crates/valence-core/src/function_object.rs",
      octet_hosted_path = "crates/valence-core/src/function_object.rs",
      compatibility_disposition = "octet-hosted-shim",
      fixture_refs = ["function-object-v1-compatible"],
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
    {
      module_or_schema = "evidence_index",
      standalone_path = "crates/valence-core/src/evidence_index.rs",
      octet_hosted_path = "crates/valence-core/src/evidence_index.rs",
      compatibility_disposition = "octet-hosted-shim",
      fixture_refs = ["evidence-index-compatible"],
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
    {
      module_or_schema = "evidence_codebase_index",
      standalone_path = "crates/valence-core/src/evidence_codebase_index.rs",
      octet_hosted_path = "crates/valence-core/src/evidence_codebase_index.rs",
      compatibility_disposition = "octet-hosted-shim",
      fixture_refs = ["evidence-codebase-index-compatible"],
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
  ],
  fixtures = [
    {
      fixture_id = "function-object-v1-compatible",
      module_or_schema = "function_object",
      expected_behavior = "Function Object v1 DTO and object hash behavior remains byte-compatible for checked fixtures",
      compatibility_hash_source = "cargo test -p cargo-octet object -- --nocapture",
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
    {
      fixture_id = "evidence-index-compatible",
      module_or_schema = "evidence_index",
      expected_behavior = "current-head evidence index render/check behavior remains byte-compatible for checked fixtures",
      compatibility_hash_source = "cargo test -p cargo-octet evidence_index -- --nocapture",
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
    {
      fixture_id = "evidence-codebase-index-compatible",
      module_or_schema = "evidence_codebase_index",
      expected_behavior = "evidence-codebase-index build/query/html behavior remains byte-compatible for checked fixtures",
      compatibility_hash_source = "cargo test -p valence-core evidence_codebase_index -- --nocapture",
      non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
    },
  ],
  migration_targets = [
    {
      consumer = "Aspen",
      current_dependency = "git package valence-core from github:OnixResearch/octet",
      target_dependency = "github:OnixResearch/valence?dir=crates/valence-core or a reviewed local ../valence/crates/valence-core override",
      validation_command = "cargo test -p cargo-octet object evidence_index -- --nocapture; cargo test -p valence-core function_object evidence_codebase_index evidence_index -- --nocapture",
      non_claim = "cutover validation does not prove behavioral correctness, external consumer compatibility, or release eligibility",
    },
  ],
  supported_claim = "cutover validation proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility",
  non_claims = ["standalone Valence cutover proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"],
} | CutoverManifest
