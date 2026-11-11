let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Identity = std.contract.from_predicate (
  fun value => std.is_string value && std.string.length value > 3 && std.string.substring 0 3 value == "b3:"
) in
let Scope = {
  paths | Array NonEmpty,
  extensions | Array NonEmpty,
  exclude_globs | Array NonEmpty,
  max_files | Positive,
  max_bytes | Positive,
} in
let RustScope = {
  build_mode | NonEmpty,
  target | NonEmpty,
  features | Array NonEmpty,
  packages | Array NonEmpty,
  unsupported_features | Array NonEmpty,
} in
let Limits = {
  wall_clock_ms | Positive,
  stdout_max_bytes | Positive,
  stderr_max_bytes | Positive,
  max_report_bytes | Positive,
  max_memory_mb | Positive,
  max_threads | Positive,
} in
let RuleSource = {
  kind | NonEmpty,
  path | String,
  identity | Identity,
  identity_source | NonEmpty,
} in
let QueryPack = {
  name | NonEmpty,
  version | NonEmpty,
  identity | Identity,
  identity_source | NonEmpty,
} in
let Database = {
  language | NonEmpty,
  build_mode | NonEmpty,
  policy | NonEmpty,
} in
let ProducerProfile = {
  profile_id | NonEmpty,
  producer | NonEmpty,
  intake_profile_id | NonEmpty,
  driver_name | NonEmpty,
  executable = { name | NonEmpty, declared_version | NonEmpty },
  supported_languages | Array NonEmpty,
  rules | RuleSource,
  query_packs | Array QueryPack,
  database | Dyn,
  scope | Scope,
  rust | RustScope,
  limits | Limits,
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-sarif-producer-profiles/v1",
  profiles = [
    {
      profile_id = "semgrep-exec-local",
      producer = "semgrep",
      intake_profile_id = "semgrep-local",
      driver_name = "Semgrep",
      executable = {
        name = "semgrep",
        declared_version = "1.180.0",
      },
      supported_languages = ["rust"],
      rules = {
        kind = "local-rule-file",
        path = "standards/sarif/semgrep-rust-rules.yml",
        identity = "b3:a98c53e249ae5a34bbedd33c481467ba889c2fd592776ebd104dcc5b87140389",
        identity_source = "checked-file-bytes",
      },
      query_packs = [],
      database = null,
      scope = {
        paths = ["cargo-octet/src", "src", "crates"],
        extensions = [".rs"],
        exclude_globs = ["**/target/**", "**/tests/fixtures/**", "**/.octet/**"],
        max_files = 20000,
        max_bytes = 67108864,
      },
      rust = {
        build_mode = "source-only",
        target = "not-selected",
        features = [],
        packages = [],
        unsupported_features = [
          "macro-expanded and proc-macro-generated items",
          "type, trait, and borrow resolution",
          "interprocedural dataflow outside the selected rule bundle",
          "cargo feature and target selection",
        ],
      },
      limits = {
        wall_clock_ms = 900000,
        stdout_max_bytes = 16777216,
        stderr_max_bytes = 16777216,
        max_report_bytes = 1048576,
        max_memory_mb = 2048,
        max_threads = 1,
      },
      non_claims = [
        "Octet executed this Semgrep scan locally against the bound source snapshot; it does not transfer Semgrep or Semgrep Cloud authority.",
        "Semgrep findings are bounded external static observations and do not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.",
        "The selected Semgrep engine, rule bundle, and supported Rust subset are explicit; syntax-only rules do not model macros, trait resolution, or interprocedural dataflow outside the bundle.",
      ],
    },
    {
      profile_id = "codeql-exec-local",
      producer = "codeql",
      intake_profile_id = "codeql-local",
      driver_name = "CodeQL",
      executable = {
        name = "codeql",
        declared_version = "2.27.2",
      },
      supported_languages = ["rust"],
      rules = {
        kind = "query-pack-set",
        path = "",
        identity = "b3:22849f283d0f21cece7bddee7c1e09a9c1b02b37cd103930ed52eb7d987e7393",
        identity_source = "declared-pin",
      },
      query_packs = [
        {
          name = "codeql/rust-queries",
          version = "0.1.46",
          identity = "b3:fd1e7991fbb379fa5cfb0180fda5fcfd0db9d35d47ae9fe4238430657945c8d5",
          identity_source = "declared-pin",
        },
      ],
      database = {
        language = "rust",
        build_mode = "none",
        policy = "reuse-or-create",
      },
      scope = {
        paths = ["cargo-octet/src", "src", "crates"],
        extensions = [".rs"],
        exclude_globs = ["**/target/**", "**/tests/fixtures/**", "**/.octet/**"],
        max_files = 20000,
        max_bytes = 67108864,
      },
      rust = {
        build_mode = "none",
        target = "not-selected",
        features = [],
        packages = [],
        unsupported_features = [
          "non-cargo build systems under build-mode none",
          "build scripts that generate Rust source before extraction",
          "type-level or runtime behavior outside the selected query pack",
          "cargo feature and target selection",
        ],
      },
      limits = {
        wall_clock_ms = 3600000,
        stdout_max_bytes = 16777216,
        stderr_max_bytes = 16777216,
        max_report_bytes = 1048576,
        max_memory_mb = 4096,
        max_threads = 2,
      },
      non_claims = [
        "Octet executed this CodeQL analysis locally against the bound source snapshot and database; it does not transfer GitHub code scanning or CodeQL pack authority.",
        "CodeQL findings are bounded external static observations and do not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.",
        "The selected CodeQL engine, query packs, and supported Rust subset are explicit; build-mode none does not model build-script-generated source or non-cargo build systems.",
      ],
    },
  ],
} | {
  schema_version | std.contract.from_predicate (fun value => value == "octet-sarif-producer-profiles/v1"),
  profiles | Array ProducerProfile,
}
