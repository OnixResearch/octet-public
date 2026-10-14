# r[impl octet.aeneas_lean_proof_rail.provider_contract]
# r[impl octet.aeneas_lean_proof_rail.semantic_reference]
let contracts = import "contracts.ncl" in
let default_timeout_ms = 30000 in
let default_log_max_bytes = 4096 in
let target_pointer_width_bits = 64 in
{
  schema_version = "octet-aeneas-lean-provider-profile/v1",
  profile_id = "octet-aeneas-lean-0.1.0-bc2eb6f",
  provider_id = "aeneas-lean",
  claim_scope = "named-theorem-over-generated-model",
  aeneas_release = "0.1.0",
  aeneas_revision = "bc2eb6f941369fa3c695e30598b5758b11e4f0de",
  aeneas_repository = "https://github.com/AeneasVerif/aeneas",
  charon_revision = "cb50ff16b9f1066b8a97dc06da704de2da2fa41c",
  charon_repository = "https://github.com/AeneasVerif/charon",
  ocaml_toolchain = "ocaml@5.2.0",
  lean_toolchain = "leanprover/lean4:v4.31.0",
  aeneas_lean_library = "AeneasVerif/aeneas/backends/lean@bc2eb6f941369fa3c695e30598b5758b11e4f0de",
  checked_source_surface = [
    "safe sequential Rust",
    "no unsafe Rust",
    "no concurrency",
    "reviewed Charon-supported subset only",
  ],
  command_argv = ["aeneas", "--backend", "lean", "--emit-json"],
  stage_commands = {
    charon_argv = ["charon", "cargo", "--preset", "aeneas", "--dest-file", "llbc.llbc"],
    aeneas_argv = ["aeneas", "-backend", "lean", "-emit-json", "-dest", ".", "-no-progress-bar"],
    lean_argv = ["lake", "build"],
  },
  limits = {
    timeout_ms = default_timeout_ms,
    stdout_max_bytes = default_log_max_bytes,
    stderr_max_bytes = default_log_max_bytes,
  },
  environment_allowlist = ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME", "MATHLIB_NO_CACHE_ON_UPDATE"],
  extraction_policy = {
    has_errors = false,
    skip_borrowck = false,
    target_triple = "x86_64-unknown-linux-gnu",
    pointer_width_bits = target_pointer_width_bits,
    endianness = "little",
    mir_start_stage = "borrowck",
    drop_policy = "include-drops",
    start_from = "crate",
    include_rules = ["src/verified"],
    exclude_rules = [],
    opaque_rules = [],
    transformation_settings = ["preserve-spans"],
    serialization_format = "llbc-json-v1",
  },
  assumption_policy = {
    external_models = [],
    axioms = [],
    requires_complete_declaration = true,
  },
  semantic_reference = {
    repository = "https://github.com/AeneasVerif/mechanized-llbc",
    revision = "8a9b8bb3823ceceb47026837d23baea0934dfb27",
    license = "Apache-2.0",
    source_sentinel = "README.md:Mechanized LLBC",
    recorded_only = true,
  },
  non_claims = [
    "does not prove Aeneas or Charon translation soundness",
    "does not prove Lean or verifier soundness",
    "does not prove whole-program correctness",
    "does not prove rustc compiler correctness",
    "does not prove external model soundness",
    "does not prove unsafe Rust or concurrent Rust",
    "does not establish release eligibility",
  ],
} | contracts.AeneasLeanProviderProfile
