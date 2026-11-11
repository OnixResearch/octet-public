let contracts = import "contracts.ncl" in
let bounded_non_claims = [
  "Clippy restriction diagnostics are compiler lint evidence and do not prove behavioral correctness, panic freedom, memory safety, runtime determinism, compiler correctness, formal proof, whole-program correctness, or release eligibility.",
  "The profile checks direct compiled targets under the recorded command and toolchain; it does not establish transitive dependency behavior or targets omitted from that command.",
] in
{
  schema_version = "octet-clippy-restriction-profiles/v1",
  profiles = [
    {
      profile_id = "production-safety",
      summary = "Reject debug macros, destructor bypass, undocumented unsafe blocks, and unsafe blocks that combine multiple operations.",
      lint_level = "deny",
      lints = [
        "dbg_macro",
        "mem_forget",
        "undocumented_unsafe_blocks",
        "multiple_unsafe_ops_per_block",
      ],
      claim_scope = "clippy-restriction-diagnostics",
      non_claims = bounded_non_claims,
    },
    {
      profile_id = "canonical-determinism",
      summary = "Apply production restrictions plus explicit byte order and deterministic hash-collection iteration requirements.",
      lint_level = "deny",
      lints = [
        "dbg_macro",
        "mem_forget",
        "undocumented_unsafe_blocks",
        "multiple_unsafe_ops_per_block",
        "host_endian_bytes",
        "iter_over_hash_type",
      ],
      claim_scope = "clippy-restriction-diagnostics",
      non_claims = bounded_non_claims,
    },
    {
      profile_id = "boolean-api-modeling",
      summary = "Co-report Clippy's excessive boolean-parameter finding independently of Octet's direct public API rule.",
      lint_level = "warn",
      lints = ["fn_params_excessive_bools"],
      claim_scope = "clippy-restriction-diagnostics",
      non_claims = bounded_non_claims,
    },
    {
      profile_id = "shell-boundary-inventory",
      summary = "Inventory process exits outside main so helpers can return status through a thin imperative shell.",
      lint_level = "warn",
      lints = ["exit"],
      claim_scope = "clippy-restriction-diagnostics",
      non_claims = bounded_non_claims,
    },
  ],
} | contracts.Catalog
