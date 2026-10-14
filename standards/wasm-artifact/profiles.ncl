let contracts = import "contracts.ncl" in
let spacewasm_support = import "spacewasm/support-projection.ncl" in
let maximum_artifact_bytes = 1048576 in
let maximum_imports = 32 in
let maximum_exports = 64 in
let maximum_memories = 1 in
let maximum_tables = 1 in
let maximum_functions = 128 in
let maximum_linear_memory_pages = 65536 in
let maximum_table_elements = 65535 in
let maximum_function_parameter_words = 255 in
let maximum_local_words_per_function = 65535 in
let spacewasm_maximum_imports = 0 in
let spacewasm_maximum_exports = 16 in
let spacewasm_maximum_functions = 4096 in
let spacewasm_maximum_memory_pages = 256 in
let spacewasm_maximum_table_elements = 1024 in
let spacewasm_maximum_function_parameter_words = 32 in
let spacewasm_maximum_local_words_per_function = 1024 in
{
  schema_version = "octet-wasm-artifact-profiles/v1",
  cohort = {
    cohort_id = "wasm-tools-0.253.0",
    identity = "b3:c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a",
    members = [
      { crate_name = "wasm-encoder", version = "0.253.0", role = "fixture-encoding" },
      { crate_name = "wasm-metadata", version = "0.253.0", role = "metadata-inspection" },
      { crate_name = "wasm-mutate", version = "0.253.0", role = "bounded-mutation" },
      { crate_name = "wasm-shrink", version = "0.253.0", role = "bounded-shrinking" },
      { crate_name = "wasm-smith", version = "0.253.0", role = "bounded-generation" },
      { crate_name = "wasmparser", version = "0.253.0", role = "validation-and-facts" },
      { crate_name = "wit-component", version = "0.253.0", role = "component-metadata" },
      { crate_name = "wit-parser", version = "0.253.0", role = "wit-interface-facts" },
      { crate_name = "wit-smith", version = "0.253.0", role = "bounded-wit-generation" },
    ],
  },
  profiles = [
    {
      profile_id = "portable-component-baseline",
      summary = "Execution-free static facts for portable core modules and components with no ambient host imports.",
      allowed_artifact_kinds = ["module", "component"],
      allowed_proposals = ["wasm1", "component-model"],
      allowed_worlds = [],
      allowed_imports = [],
      allowed_exports = [],
      required_exports = [],
      allow_any_exports = true,
      resource_limits = {
        max_artifact_bytes = maximum_artifact_bytes,
        max_imports = maximum_imports,
        max_exports = maximum_exports,
        max_memories = maximum_memories,
        max_tables = maximum_tables,
        max_functions = maximum_functions,
        max_memory_initial_pages = maximum_linear_memory_pages,
        max_memory_maximum_pages = maximum_linear_memory_pages,
        max_table_initial_elements = maximum_table_elements,
        max_table_maximum_elements = maximum_table_elements,
        max_function_parameter_words = maximum_function_parameter_words,
        max_local_words_per_function = maximum_local_words_per_function,
      },
      allow_memory_grow = false,
      allow_start = false,
      duplicate_custom_sections = "reject",
      producer_metadata = "recorded-only",
      severity = {
        malformed = "error",
        disallowed_kind = "error",
        disallowed_import = "error",
        disallowed_world = "error",
        resource_limit = "error",
        growth = "error",
        start = "error",
        duplicate_metadata = "error",
        abi_mismatch = "error",
        external_evidence = "error",
        cohort_drift = "error",
      },
      verification_role = "recorded_only",
      non_claims = [
        "Wasm artifact evidence records bounded static facts and does not prove execution correctness",
        "Matching normalized interfaces do not prove semantic equivalence or identify one implementation",
        "Static validation does not prove sandbox effectiveness, capability authority, provenance truth, release eligibility, or whole-system safety",
        "Producer metadata is recorded input and is not provenance truth",
      ],
      target_runtime = null,
    },
    {
      profile_id = "spacewasm-mvp",
      summary = "Hostless core-module ABI and static resource subset reviewed against NASA SpaceWasm revision e24cf09355a90497148eb5029fdb8e3400bd63e3.",
      allowed_artifact_kinds = ["module"],
      allowed_proposals = ["wasm1"],
      allowed_worlds = [],
      allowed_imports = [],
      allowed_exports = ["module::run::Func"],
      required_exports = ["module::run::Func"],
      allow_any_exports = false,
      resource_limits = {
        max_artifact_bytes = maximum_artifact_bytes,
        max_imports = spacewasm_maximum_imports,
        max_exports = spacewasm_maximum_exports,
        max_memories = maximum_memories,
        max_tables = maximum_tables,
        max_functions = spacewasm_maximum_functions,
        max_memory_initial_pages = spacewasm_maximum_memory_pages,
        max_memory_maximum_pages = spacewasm_maximum_memory_pages,
        max_table_initial_elements = spacewasm_maximum_table_elements,
        max_table_maximum_elements = spacewasm_maximum_table_elements,
        max_function_parameter_words = spacewasm_maximum_function_parameter_words,
        max_local_words_per_function = spacewasm_maximum_local_words_per_function,
      },
      allow_memory_grow = false,
      allow_start = false,
      duplicate_custom_sections = "reject",
      producer_metadata = "recorded-only",
      severity = {
        malformed = "error",
        disallowed_kind = "error",
        disallowed_import = "error",
        disallowed_world = "error",
        resource_limit = "error",
        growth = "error",
        start = "error",
        duplicate_metadata = "error",
        abi_mismatch = "error",
        external_evidence = "error",
        cohort_drift = "error",
      },
      verification_role = "boundary",
      non_claims = [
        "Wasm artifact evidence records bounded static facts and does not prove execution correctness",
        "SpaceWasm profile conformance does not prove SpaceWasm decode, instantiation, or execution success",
        "Static declarations do not prove configured allocator, code-page, control-frame, fuel, host-import, or stack capacity",
        "The hostless run export is an Octet review profile and not a universal SpaceWasm ABI",
        "Profile conformance does not prove memory safety, semantic equivalence, sandbox effectiveness, flight qualification, production readiness, or release eligibility",
      ],
      target_runtime = {
        cohort_identity = "b3:7c537eef7bff66d0a90508715de08de79c9aaac23352e1007a6d47e7fb4674a2",
        support_projection = spacewasm_support,
      },
    },
  ],
} | contracts.Registry
