let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let MaxMutants = 2048 in
let MaxReportBytes = 16777216 in
let MaxArtifactCount = 4096 in
let MaxArtifactBytes = 67108864 in
let MaxLogBytes = 1048576 in
let TimeoutMilliseconds = 900000 in
let PollMilliseconds = 25 in
let TeardownMilliseconds = 5000 in
let Profile = {
  profile_id | NonEmpty,
  provider | {
    package | std.contract.from_predicate (fun value => value == "cargo-mutants"),
    version | std.contract.from_predicate (fun value => value == "27.0.0"),
    nixpkgs_revision | std.contract.from_predicate (fun value => value == "b3d51a0365f6695e7dd5cdf3e180604530ed33b4"),
    closure_identity | NonEmpty,
  },
  rust_toolchain | std.contract.from_predicate (fun value => value == "nightly-2026-03-21"),
  scope | {
    package | std.contract.from_predicate (fun value => value == "octet-lint-metadata"),
    target | std.contract.from_predicate (fun value => value == "lib"),
    all_features | std.contract.from_predicate (fun value => value == true),
    test_selection | std.contract.from_predicate (fun value => value == "library-tests"),
  },
  command_argv | Array NonEmpty,
  output_root | std.contract.from_predicate (fun value => value == ".octet/mutants-output"),
  native_outcomes_path | std.contract.from_predicate (fun value => value == ".octet/mutants-output/mutants.out/outcomes.json"),
  native_inventory_path | std.contract.from_predicate (fun value => value == ".octet/mutants-output/mutants.out/mutants.json"),
  normalization_version | std.contract.from_predicate (fun value => value == "octet-cargo-mutants-normalization/v1"),
  source_manifest_schema | std.contract.from_predicate (fun value => value == "octet-source-manifest/v1"),
  outcome_policy | {
    caught | std.contract.from_predicate (fun value => value == "pass"),
    missed | std.contract.from_predicate (fun value => value == "fail"),
    timeout | std.contract.from_predicate (fun value => value == "manual_review"),
    unviable | std.contract.from_predicate (fun value => value == "manual_review"),
    not_run | std.contract.from_predicate (fun value => value == "fail"),
    tool_failure | std.contract.from_predicate (fun value => value == "fail"),
  },
  retain_logs | std.contract.from_predicate (fun value => value == true),
  retain_diffs | std.contract.from_predicate (fun value => value == true),
  max_mutants | Positive,
  max_report_bytes | Positive,
  max_artifact_count | Positive,
  max_artifact_bytes | Positive,
  max_log_bytes | Positive,
  timeout_ms | Positive,
  poll_interval_ms | Positive,
  teardown_timeout_ms | Positive,
  evidence_tier | std.contract.from_predicate (fun value => value == "transcript_assertion"),
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-mutation-profiles/v1",
  profiles = [
    {
      profile_id = "cargo-mutants-octet-metadata",
      provider = {
        package = "cargo-mutants",
        version = "27.0.0",
        nixpkgs_revision = "b3d51a0365f6695e7dd5cdf3e180604530ed33b4",
        closure_identity = "nixpkgs:b3d51a0365f6695e7dd5cdf3e180604530ed33b4#cargo-mutants-27.0.0",
      },
      rust_toolchain = "nightly-2026-03-21",
      scope = {
        package = "octet-lint-metadata",
        target = "lib",
        all_features = true,
        test_selection = "library-tests",
      },
      command_argv = [
        "cargo",
        "mutants",
        "--no-times",
        "--colors",
        "never",
        "--timeout",
        "60",
        "--build-timeout",
        "120",
        "--jobs",
        "1",
        "--output",
        ".octet/mutants-output",
        "--package",
        "octet-lint-metadata",
        "--all-features",
      ],
      output_root = ".octet/mutants-output",
      native_outcomes_path = ".octet/mutants-output/mutants.out/outcomes.json",
      native_inventory_path = ".octet/mutants-output/mutants.out/mutants.json",
      normalization_version = "octet-cargo-mutants-normalization/v1",
      source_manifest_schema = "octet-source-manifest/v1",
      outcome_policy = {
        caught = "pass",
        missed = "fail",
        timeout = "manual_review",
        unviable = "manual_review",
        not_run = "fail",
        tool_failure = "fail",
      },
      retain_logs = true,
      retain_diffs = true,
      max_mutants = MaxMutants,
      max_report_bytes = MaxReportBytes,
      max_artifact_count = MaxArtifactCount,
      max_artifact_bytes = MaxArtifactBytes,
      max_log_bytes = MaxLogBytes,
      timeout_ms = TimeoutMilliseconds,
      poll_interval_ms = PollMilliseconds,
      teardown_timeout_ms = TeardownMilliseconds,
      evidence_tier = "transcript_assertion",
      non_claims = [
        "Mutation outcomes sample test-suite sensitivity and do not prove test completeness or source correctness.",
        "Survived, timed-out, unviable, not-run, and tool-failure outcomes remain visible and are not formal proof.",
      ],
    },
  ],
} | {
  schema_version | std.contract.from_predicate (fun value => value == "octet-mutation-profiles/v1"),
  profiles | Array Profile,
}
