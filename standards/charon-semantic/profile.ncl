let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let HexDigest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let B3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let Revision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let Reference = {
  repository | NonEmptyString,
  revision | Revision,
  license | std.contract.Equal "Apache-2.0",
  source_sentinel | NonEmptyString,
  use | NonEmptyString,
  recorded_only | std.contract.Equal true,
} in
let SelectedItem = {
  qualified_name | NonEmptyString,
  kind | std.contract.Equal "function",
  source_file | NonEmptyString,
  source_identity | B3Identity,
  begin_line | PositiveNumber,
  end_line | PositiveNumber,
} in
let target_pointer_width_bytes = 8 in
let provider_timeout_ms = 30000 in
let bounded_log_bytes = 4096 in
let snapshot_mebibytes = 8 in
let bytes_per_kibibyte = 1024 in
let kibibytes_per_mebibyte = 1024 in
let configured_snapshot_max_bytes = snapshot_mebibytes * bytes_per_kibibyte * kibibytes_per_mebibyte in
let configured_item_max_count = 32 in
let configured_edge_max_count = 128 in
let Profile = {
  schema_version | std.contract.Equal "octet-charon-semantic-profile/v1",
  profile_id | std.contract.Equal "octet-charon-semantic-safe-add-v1",
  provider | {
    package | std.contract.Equal "octet-aeneas-lean-provider-closure",
    profile_id | std.contract.Equal "octet-aeneas-lean-0.1.0-bc2eb6f",
    profile_blake3 | HexDigest,
    compatibility_matrix_id | std.contract.Equal "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
    compatibility_matrix_blake3 | HexDigest,
    charon_revision | Revision,
    executable_env | std.contract.Equal "OCTET_AENEAS_CHARON",
    network_sandbox_env | std.contract.Equal "OCTET_AENEAS_NETWORK_SANDBOX",
  },
  source | {
    root | NonEmptyString,
    crate_name | std.contract.Equal "real_fixture",
    selected_items | Array SelectedItem,
    broad_selection_allowed | std.contract.Equal false,
  },
  extraction | {
    command_argv | Array NonEmptyString,
    preset | std.contract.Equal "Aeneas",
    has_errors | std.contract.Equal false,
    skip_borrowck | std.contract.Equal false,
    borrow_check_preserved | std.contract.Equal true,
    target_triple | std.contract.Equal "x86_64-unknown-linux-gnu",
    pointer_width_bytes | std.contract.Equal target_pointer_width_bytes,
    endianness | std.contract.Equal "little",
    serialization_format | std.contract.Equal "llbc-json-v1",
    environment_allowlist | Array NonEmptyString,
  },
  fact_vocabulary | Array NonEmptyString,
  effect_candidate_vocabulary | Array NonEmptyString,
  limits | {
    timeout_ms | PositiveNumber,
    stdout_max_bytes | PositiveNumber,
    stderr_max_bytes | PositiveNumber,
    snapshot_max_bytes | PositiveNumber,
    item_max_count | PositiveNumber,
    edge_max_count | PositiveNumber,
  },
  artifact_roles | Array NonEmptyString,
  references | Array Reference,
  verification_role | std.contract.Equal "recorded_only",
  non_claims | Array NonEmptyString,
} in
{
  schema_version = "octet-charon-semantic-profile/v1",
  profile_id = "octet-charon-semantic-safe-add-v1",
  provider = {
    package = "octet-aeneas-lean-provider-closure",
    profile_id = "octet-aeneas-lean-0.1.0-bc2eb6f",
    profile_blake3 = "cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625",
    compatibility_matrix_id = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
    compatibility_matrix_blake3 = "403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5",
    charon_revision = "cb50ff16b9f1066b8a97dc06da704de2da2fa41c",
    executable_env = "OCTET_AENEAS_CHARON",
    network_sandbox_env = "OCTET_AENEAS_NETWORK_SANDBOX",
  },
  source = {
    root = "standards/proof-rails/aeneas-lean/fixtures/valid/real_fixture",
    crate_name = "real_fixture",
    selected_items = [
      {
        qualified_name = "real_fixture::safe_add",
        kind = "function",
        source_file = "src/lib.rs",
        source_identity = "b3:fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
        begin_line = 1,
        end_line = 3,
      },
    ],
    broad_selection_allowed = false,
  },
  extraction = {
    command_argv = [
      "charon",
      "cargo",
      "--preset",
      "aeneas",
      "--dest-file",
      "snapshot.llbc",
      "--start-from",
      "real_fixture::safe_add",
    ],
    preset = "Aeneas",
    has_errors = false,
    skip_borrowck = false,
    borrow_check_preserved = true,
    target_triple = "x86_64-unknown-linux-gnu",
    pointer_width_bytes = target_pointer_width_bytes,
    endianness = "little",
    serialization_format = "llbc-json-v1",
    environment_allowlist = ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME"],
  },
  fact_vocabulary = [
    "selected-item",
    "source-span",
    "local-call",
    "item-dependency",
    "trait-resolution",
    "control-flow-shape",
    "call-cycle",
    "opacity",
    "unsupported-marker",
    "target-metadata",
  ],
  effect_candidate_vocabulary = ["panic", "drop", "unsafe-boundary", "unknown-call"],
  limits = {
    timeout_ms = provider_timeout_ms,
    stdout_max_bytes = bounded_log_bytes,
    stderr_max_bytes = bounded_log_bytes,
    snapshot_max_bytes = configured_snapshot_max_bytes,
    item_max_count = configured_item_max_count,
    edge_max_count = configured_edge_max_count,
  },
  artifact_roles = [
    "raw-charon-snapshot",
    "normalized-semantic-projection",
    "bounded-semantic-analysis",
    "source-token-comparison",
    "recorded-only-receipt",
  ],
  references = [
    {
      repository = "https://github.com/AeneasVerif/mechanized-llbc",
      revision = "8a9b8bb3823ceceb47026837d23baea0934dfb27",
      license = "Apache-2.0",
      source_sentinel = "README.md:Mechanized LLBC",
      use = "semantic-foundation review corpus only",
      recorded_only = true,
    },
    {
      repository = "https://github.com/AeneasVerif/charon-rudra",
      revision = "0923bf7f21d4eb234d14bce86afc9690d0a4d00e",
      license = "Apache-2.0",
      source_sentinel = "src/main.rs:crate::rudra::lib::analyze",
      use = "historical analysis-over-Charon fixture design only",
      recorded_only = true,
    },
  ],
  verification_role = "recorded_only",
  non_claims = [
    "does not prove Charon or rustc correctness",
    "does not prove complete Rust support or complete runtime reachability",
    "does not prove effect absence, semantic equivalence, or whole-program correctness",
    "does not produce formal-proof, property, or release-ready evidence",
    "does not transfer mechanized-llbc or charon-rudra proof, implementation, or analysis authority",
    "does not make Valence or Kamacite a compiler IR or semantic-analysis authority",
  ],
} | Profile
