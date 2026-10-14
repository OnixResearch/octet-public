let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let BasisPointScale = 10000 in
let PrecisionDigits = 2 in
let BasisPoints = std.contract.from_predicate (fun value => std.is_number value && value >= 0 && value <= BasisPointScale) in
let MaxReportBytes = 16777216 in
let MaxFiles = 10000 in
let MaxPathBytes = 240 in
let MaxLogBytes = 1048576 in
let TimeoutMilliseconds = 300000 in
let PollMilliseconds = 25 in
let TeardownMilliseconds = 5000 in
let LineThreshold = 8000 in
let BranchThreshold = 7000 in
let FunctionThreshold = 8000 in
let RegionThreshold = 8000 in
let Profile = {
  profile_id | NonEmpty,
  provider | {
    package | std.contract.from_predicate (fun value => value == "cargo-llvm-cov"),
    version | std.contract.from_predicate (fun value => value == "0.8.3"),
    nixpkgs_revision | std.contract.from_predicate (fun value => value == "b3d51a0365f6695e7dd5cdf3e180604530ed33b4"),
    closure_identity | NonEmpty,
  },
  rust_toolchain | std.contract.from_predicate (fun value => value == "nightly-2026-03-21"),
  llvm_tools_component | std.contract.from_predicate (fun value => value == "llvm-tools-preview"),
  scope | {
    package | std.contract.from_predicate (fun value => value == "cargo-octet"),
    target | std.contract.from_predicate (fun value => value == "lib"),
    all_features | std.contract.from_predicate (fun value => value == true),
    test_selection | std.contract.from_predicate (fun value => value == "library-tests"),
  },
  metrics | Array NonEmpty,
  command_argv | Array NonEmpty,
  current_report_path | std.contract.from_predicate (fun value => value == ".octet/coverage/current-native.json"),
  normalization_version | std.contract.from_predicate (fun value => value == "octet-coverage-normalization/v1"),
  source_scope_identity | NonEmpty,
  path_policy_identity | std.contract.from_predicate (fun value => value == "workspace-relative-remapped-v1"),
  threshold_basis_points | {
    lines | BasisPoints,
    branches | BasisPoints,
    functions | BasisPoints,
    regions | BasisPoints,
  },
  regression_policy | std.contract.from_predicate (fun value => value == "deny_decrease"),
  denominator_policy | std.contract.from_predicate (fun value => value == "require_equal"),
  rounding_policy | std.contract.from_predicate (fun value => value == "floor"),
  precision_digits | std.contract.from_predicate (fun value => value == PrecisionDigits),
  max_report_bytes | Positive,
  max_files | Positive,
  max_path_bytes | Positive,
  max_log_bytes | Positive,
  timeout_ms | Positive,
  poll_interval_ms | Positive,
  teardown_timeout_ms | Positive,
  evidence_tier | std.contract.from_predicate (fun value => value == "transcript_assertion"),
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-coverage-profiles/v1",
  profiles = [
    {
      profile_id = "cargo-llvm-cov-octet-lib",
      provider = {
        package = "cargo-llvm-cov",
        version = "0.8.3",
        nixpkgs_revision = "b3d51a0365f6695e7dd5cdf3e180604530ed33b4",
        closure_identity = "nixpkgs:b3d51a0365f6695e7dd5cdf3e180604530ed33b4#cargo-llvm-cov-0.8.3",
      },
      rust_toolchain = "nightly-2026-03-21",
      llvm_tools_component = "llvm-tools-preview",
      scope = {
        package = "cargo-octet",
        target = "lib",
        all_features = true,
        test_selection = "library-tests",
      },
      metrics = ["lines", "branches", "functions", "regions"],
      command_argv = [
        "cargo",
        "llvm-cov",
        "--json",
        "--summary-only",
        "--remap-path-prefix",
        "--output-path",
        ".octet/coverage/current-native.json",
        "--package",
        "cargo-octet",
        "--lib",
        "--all-features",
        "--locked",
        "--offline",
      ],
      current_report_path = ".octet/coverage/current-native.json",
      normalization_version = "octet-coverage-normalization/v1",
      source_scope_identity = "octet:cargo-octet:lib:all-features:library-tests",
      path_policy_identity = "workspace-relative-remapped-v1",
      threshold_basis_points = {
        lines = LineThreshold,
        branches = BranchThreshold,
        functions = FunctionThreshold,
        regions = RegionThreshold,
      },
      regression_policy = "deny_decrease",
      denominator_policy = "require_equal",
      rounding_policy = "floor",
      precision_digits = PrecisionDigits,
      max_report_bytes = MaxReportBytes,
      max_files = MaxFiles,
      max_path_bytes = MaxPathBytes,
      max_log_bytes = MaxLogBytes,
      timeout_ms = TimeoutMilliseconds,
      poll_interval_ms = PollMilliseconds,
      teardown_timeout_ms = TeardownMilliseconds,
      evidence_tier = "transcript_assertion",
      non_claims = [
        "Coverage counts show instrumented execution only and do not prove behavioral correctness or path correctness.",
        "Coverage evidence does not prove test completeness, formal proof, whole-program correctness, or release eligibility.",
      ],
    },
  ],
} | {
  schema_version | std.contract.from_predicate (fun value => value == "octet-coverage-profiles/v1"),
  profiles | Array Profile,
}
