let SourceRoot = {
  build_root | String,
  repository_root | String,
}
in
let ResourceLimits = {
  max_binary_bytes | Number,
  max_debug_bytes | Number,
  max_sections | Number,
  max_units | Number,
  max_symbols | Number,
  max_ranges | Number,
  max_line_rows | Number,
  max_frames | Number,
  max_path_bytes | Number,
  max_rows | Number,
}
in
let Severity = {
  malformed | String,
  unsupported | String,
  wrong_companion | String,
  unmapped_path | String,
  stale_span | String,
  ambiguous_function | String,
  resource_limit | String,
}
in
let Profile = {
  profile_id | String,
  summary | String,
  object_format | String,
  architecture | String,
  debug_format | String,
  required_sections | Array String,
  source_roots | Array SourceRoot,
  unmapped_path_policy | String,
  strict_linkage | Bool,
  companion_match | String,
  resource_limits | ResourceLimits,
  severity | Severity,
  verification_role | String,
  non_claims | Array String,
}
in
{
  schema_version = "octet-binary-debug-map-profiles/v1",
  profiles = [
    {
      profile_id = "elf-dwarf-v1",
      summary = "Bounded ELF/DWARF symbol-to-source linkage over supplied bytes with explicit source-root mappings.",
      object_format = "elf",
      architecture = "x86_64",
      debug_format = "dwarf",
      required_sections = [".debug_abbrev", ".debug_info", ".debug_line"],
      source_roots = [
        {
          build_root = "/build/octet-binary-debug-map",
          repository_root = "crates/binary-debug-map-fixture",
        },
      ],
      unmapped_path_policy = "fail",
      strict_linkage = true,
      companion_match = "build-id",
      resource_limits = {
        max_binary_bytes = 8388608,
        max_debug_bytes = 8388608,
        max_sections = 128,
        max_units = 16,
        max_symbols = 4096,
        max_ranges = 8192,
        max_line_rows = 65536,
        max_frames = 4096,
        max_path_bytes = 512,
        max_rows = 4096,
      },
      severity = {
        malformed = "error",
        unsupported = "error",
        wrong_companion = "error",
        unmapped_path = "error",
        stale_span = "error",
        ambiguous_function = "error",
        resource_limit = "error",
      },
      verification_role = "recorded_only",
      non_claims = [
        "Binary debug-map evidence records bounded ELF/DWARF linkage only and does not prove source-to-binary semantic equivalence, compiler correctness, or behavioral correctness",
        "Demangled symbol text is display metadata and is not a canonical function identity",
        "Raw binary-local addresses are not stable cross-build function identities",
        "Binary debug-map evidence is optional and not part of starter cargo octet check",
        "Binary debug-map evidence does not prove requirement satisfaction or release eligibility",
      ],
    },
  ],
}
