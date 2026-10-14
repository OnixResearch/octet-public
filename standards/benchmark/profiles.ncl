let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let SampleCount = 5 in
let MinimumSamples = 5 in
let PracticalThresholdBasisPoints = 500 in
let UncertaintyThresholdBasisPoints = 2000 in
let MaximumSamples = 16 in
let MaximumLogBytes = 1048576 in
let MaximumArtifactBytes = 16777216 in
let TimeoutMilliseconds = 60000 in
let PollMilliseconds = 25 in
let TeardownMilliseconds = 5000 in
let WorkloadIdentity = "b3:9f5b11ac410e6d1d48a0c25e8a90bc8b08796fe38c09ab95cc2d60773f44924b" in
let SourceStateIdentity = "b3:d2e28ced2bfa8d6c87523d6465fdf711f3385fac77435760f2d28c2efc96b93b" in
let EnvironmentIdentity = "b3:b16af370254215b0678196220770ac24285fe2e88db012057aa9ab1d5eaf900f" in
let CompilerIdentity = "b3:91747dc041e4e5768b1008ae54c6fe045cecf4bceef63551e31f051312184809" in
let Command = ["cargo", "metadata", "--no-deps", "--format-version", "1"] in
let EnvironmentFacts = ["host_os", "host_arch", "kernel_identity", "cpu_identity", "governor", "load_class", "rust_toolchain", "nix_closure_identity", "isolation"] in
let Workload = {
  workload_id | NonEmpty,
  workload_identity | NonEmpty,
  command_id | std.contract.from_predicate (fun value => value == "cargo-metadata-no-deps"),
  command_argv | Array NonEmpty,
  mode | std.contract.from_predicate (fun value => std.array.elem value ["cold", "warm", "no_change", "incremental"]),
  source_state_identity | NonEmpty,
  environment_identity | NonEmpty,
  required_environment_facts | Array NonEmpty,
  sample_count | Positive,
  required_metrics | Array NonEmpty,
} in
let Profile = {
  profile_id | NonEmpty,
  provider | {
    package | std.contract.from_predicate (fun value => value == "cargo-octet"),
    version | std.contract.from_predicate (fun value => value == "0.1.0"),
    nixpkgs_revision | std.contract.from_predicate (fun value => value == "b3d51a0365f6695e7dd5cdf3e180604530ed33b4"),
    closure_identity | NonEmpty,
  },
  rust_toolchain | std.contract.from_predicate (fun value => value == "nightly-2026-03-21"),
  workloads | Array Workload,
  signal_corpus | Array {
    corpus_id | NonEmpty,
    source_kind | std.contract.from_predicate (fun value => std.array.elem value ["synthetic", "cross_project"]),
    source_repository | NonEmpty,
    source_revision | NonEmpty,
    representativeness_boundary | NonEmpty,
  },
  resource_observation | {
    allocation_input_path | NonEmpty,
    llvm_input_path | NonEmpty,
    compiler_identity | NonEmpty,
    target | NonEmpty,
    optimization_profile | NonEmpty,
  },
  comparison | {
    summary | std.contract.from_predicate (fun value => value == "median"),
    practical_threshold_basis_points | Positive,
    uncertainty_threshold_basis_points | Positive,
    minimum_samples | Positive,
  },
  limits | {
    max_samples | Positive,
    max_log_bytes | Positive,
    max_artifact_bytes | Positive,
    timeout_ms | Positive,
    poll_interval_ms | Positive,
    teardown_timeout_ms | Positive,
  },
  evidence_tier | std.contract.from_predicate (fun value => value == "transcript_assertion"),
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-benchmark-profiles/v1",
  profiles = [
    {
      profile_id = "octet-metadata-benchmark",
      provider = {
        package = "cargo-octet",
        version = "0.1.0",
        nixpkgs_revision = "b3d51a0365f6695e7dd5cdf3e180604530ed33b4",
        closure_identity = "nixpkgs:b3d51a0365f6695e7dd5cdf3e180604530ed33b4#cargo-octet-0.1.0",
      },
      rust_toolchain = "nightly-2026-03-21",
      workloads = std.array.map (fun selected_mode => {
        workload_id = "octet-metadata-" ++ selected_mode,
        workload_identity = WorkloadIdentity,
        command_id = "cargo-metadata-no-deps",
        command_argv = Command,
        mode = selected_mode,
        source_state_identity = SourceStateIdentity,
        environment_identity = EnvironmentIdentity,
        required_environment_facts = EnvironmentFacts,
        sample_count = SampleCount,
        required_metrics = ["wall_duration_ns", "output_bytes"],
      }) ["cold", "warm", "no_change", "incremental"],
      signal_corpus = [
        {
          corpus_id = "synthetic-octet-benchmark-cases",
          source_kind = "synthetic",
          source_repository = "github:OnixResearch/octet",
          source_revision = "b3:8559f610646662c45ec8acc2fd0493a8fa25a1b97b7515609540606d982ab4a1",
          representativeness_boundary = "Selected synthetic cases exercise classifications and do not represent all Rust defects.",
        },
        {
          corpus_id = "bounded-exec-cross-project-cases",
          source_kind = "cross_project",
          source_repository = "github:OnixResearch/bounded-exec",
          source_revision = "44eb74eb639194665ee820562d5b2ce2c3ff6a94",
          representativeness_boundary = "The immutable Bounded Exec selection covers one command-shell corpus and does not represent all consumers.",
        },
      ],
      resource_observation = {
        allocation_input_path = ".octet/benchmark-allocation-observations.json",
        llvm_input_path = ".octet/benchmark-copy-sites.ll",
        compiler_identity = CompilerIdentity,
        target = "x86_64-unknown-linux-gnu",
        optimization_profile = "release",
      },
      comparison = {
        summary = "median",
        practical_threshold_basis_points = PracticalThresholdBasisPoints,
        uncertainty_threshold_basis_points = UncertaintyThresholdBasisPoints,
        minimum_samples = MinimumSamples,
      },
      limits = {
        max_samples = MaximumSamples,
        max_log_bytes = MaximumLogBytes,
        max_artifact_bytes = MaximumArtifactBytes,
        timeout_ms = TimeoutMilliseconds,
        poll_interval_ms = PollMilliseconds,
        teardown_timeout_ms = TeardownMilliseconds,
      },
      evidence_tier = "transcript_assertion",
      non_claims = [
        "Benchmark evidence covers only the selected workload, command, mode, environment, and retained samples.",
        "Timing, allocation, copy-site, and signal observations do not prove correctness, universal performance, or release eligibility.",
      ],
    },
  ],
} | {
  schema_version | std.contract.from_predicate (fun value => value == "octet-benchmark-profiles/v1"),
  profiles | Array Profile,
}
