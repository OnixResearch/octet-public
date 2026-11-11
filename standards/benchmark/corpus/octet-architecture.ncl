# Architecture policy used by the cross-tool benchmark corpus.
#
# The policy is intentionally small: it classifies the fixture packages into
# core, application-shell, outbound-adapter, and composition-root roles and
# forbids application shells from constructing adapter types directly. The
# corpus cases are selected source files; a conforming checker reports the
# violating files and stays silent for the compliant files.
let contracts = import "../../project-architecture/octet-self-architecture.contracts.ncl" in
{
  schema_version = "octet-project-architecture-policy/v1",
  project_id = "octet-benchmark-architecture-corpus",
  mode = "inventory",
  production = {
    packages = [
      "app",
      "bin_target",
      "core-api",
      "empty_scope",
      "facade",
      "gated",
      "infra-api",
      "traits",
    ],
    source_scopes = [
      "crates/app/src",
      "crates/bin_target/src",
      "crates/core-api/src",
      "crates/empty_scope/src",
      "crates/facade/src",
      "crates/gated/src",
      "crates/infra-api/src",
      "crates/traits/src",
    ],
    targets = [{
      triple = "x86_64-unknown-linux-gnu",
      features = [],
      default_features = true,
    }],
  },
  roles = [
    { scope_kind = "package", scope = "app", role = "application-shell" },
    { scope_kind = "package", scope = "bin_target", role = "composition-root" },
    { scope_kind = "package", scope = "core-api", role = "domain-core" },
    { scope_kind = "package", scope = "empty_scope", role = "domain-core" },
    { scope_kind = "package", scope = "facade", role = "application-shell" },
    { scope_kind = "package", scope = "gated", role = "application-shell" },
    { scope_kind = "package", scope = "infra-api", role = "outbound-adapter" },
    { scope_kind = "package", scope = "traits", role = "application-shell" },
    { scope_kind = "source", scope = "crates/app/src", role = "application-shell" },
    { scope_kind = "source", scope = "crates/bin_target/src", role = "composition-root" },
    { scope_kind = "source", scope = "crates/core-api/src", role = "domain-core" },
    { scope_kind = "source", scope = "crates/empty_scope/src", role = "domain-core" },
    { scope_kind = "source", scope = "crates/facade/src", role = "application-shell" },
    { scope_kind = "source", scope = "crates/gated/src", role = "application-shell" },
    { scope_kind = "source", scope = "crates/infra-api/src", role = "outbound-adapter" },
    { scope_kind = "source", scope = "crates/traits/src", role = "application-shell" },
  ],
  dependency_rules = [
    {
      caller_role = "domain-core",
      allowed_callee_roles = ["domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "application-shell",
      allowed_callee_roles = ["application-shell", "domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "outbound-adapter",
      allowed_callee_roles = ["outbound-adapter", "application-shell", "domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "composition-root",
      allowed_callee_roles = [
        "composition-root",
        "application-shell",
        "outbound-adapter",
        "domain-core",
      ],
      edge_kinds = ["normal", "build"],
    },
  ],
  provider_classifications = [],
  ports = [],
  capability_classifications = [
    {
      family = "filesystem",
      core_handling = "explicit-fact",
      executor_role = "outbound-adapter",
      rationale = "the corpus records storage access as an adapter-only effect",
    },
  ],
  outbound_effects = [],
  core_no_std = [],
  admission = null,
  waivers = [],
  collection_limits = {
    max_units = 32,
    max_shards = 32,
    max_modules = 4096,
    max_items = 65536,
    max_types = 65536,
    max_traits = 8192,
    max_implementations = 16384,
    max_signatures = 65536,
    max_calls = 262144,
    max_constructors = 65536,
    max_effects = 65536,
    max_expansions = 65536,
    max_configurations = 8192,
    max_type_shapes = 65536,
    max_function_safety_facts = 65536,
    max_value_provenance_facts = 262144,
    max_value_flow_bodies = 65536,
    max_value_flow_steps = 1048576,
    max_value_flow_rounds = 64,
    max_unknowns = 65536,
    max_spans = 262144,
    max_span_bytes = 1048576,
    max_shard_bytes = 67108864,
    max_artifact_bytes = 268435456,
  },
  non_claims = [
    "The corpus is a bounded architecture fixture and does not represent all Rust projects.",
    "The policy is a benchmark fixture and grants no release or consumer authority.",
  ],
} | contracts.ProjectArchitecturePolicy | contracts.ProjectArchitecturePolicyRules
