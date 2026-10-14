let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Canary = {
  canary_id | NonEmptyString,
  category | NonEmptyString,
  execution | OneOf ["pure", "live"],
  expected | OneOf ["pass", "reject"],
  test_function | NonEmptyString,
  rationale | NonEmptyString,
} in
let Matrix = {
  schema_version | OneOf ["octet-aeneas-lean-compatibility-matrix/v1"],
  matrix_id | NonEmptyString,
  provider_id | OneOf ["aeneas-lean"],
  reviewed_profile | NonEmptyString,
  upgrade_dimensions | NonEmptyArray | Array NonEmptyString,
  canaries | NonEmptyArray | Array Canary,
} in
{
  schema_version = "octet-aeneas-lean-compatibility-matrix/v1",
  matrix_id = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
  provider_id = "aeneas-lean",
  reviewed_profile = "standards/proof-rails/aeneas-lean/generated/profile.json",
  upgrade_dimensions = [
    "aeneas",
    "charon",
    "rustc",
    "lean",
    "aeneas_library",
    "schema",
    "provider_profile",
  ],
  canaries = [
    {
      canary_id = "live-positive-provider",
      category = "positive",
      execution = "live",
      expected = "pass",
      test_function = "live_provider_executes_real_positive_fixture",
      rationale = "the pinned Charon, Aeneas, local Lake closure, generated model, and named theorem execute successfully with network denied",
    },
    {
      canary_id = "live-false-theorem",
      category = "false-vacuous",
      execution = "live",
      expected = "reject",
      test_function = "live_provider_rejects_real_false_fixture",
      rationale = "the pinned live provider rejects the false theorem without timeout or network access",
    },
    {
      canary_id = "live-vacuous-theorem",
      category = "false-vacuous",
      execution = "live",
      expected = "reject",
      test_function = "live_provider_rejects_real_vacuous_fixture",
      rationale = "the pinned live provider rejects a vacuous theorem during adequacy preflight rather than promoting Lean acceptance",
    },
    {
      canary_id = "positive-provider-envelope",
      category = "positive",
      execution = "pure",
      expected = "pass",
      test_function = "aeneas_payload_and_envelope_accept_complete_typed_contract",
      rationale = "the reviewed provider contract and complete stage chain remain admissible",
    },
    {
      canary_id = "false-and-vacuous-theorem",
      category = "false-vacuous",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_formal_proof_rejects_false_and_vacuous_theorems",
      rationale = "false and True-style theorem selections must not promote property evidence",
    },
    {
      canary_id = "approved-theorem-statement",
      category = "theorem-approval",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_theorem_statement_approval_accepts_match_and_rejects_missing_or_mismatched",
      rationale = "the measured theorem bytes must match an independently configured BLAKE3 rather than self-approving at receipt construction",
    },
    {
      canary_id = "partial-output-stage-link",
      category = "partial-output",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_payload_rejects_broken_stage_link_before_provider_admission",
      rationale = "partial or stale stage output must stop before theorem admission",
    },
    {
      canary_id = "borrow-check-bypass-and-option-drift",
      category = "borrow-check-bypass",
      execution = "pure",
      expected = "reject",
      test_function = "charon_extraction_contract_rejects_drift_and_weakened_options",
      rationale = "borrow-check bypass and weakened extraction options must fail closed",
    },
    {
      canary_id = "opacity-model-trust-closure",
      category = "opacity-model",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_formal_proof_rejects_missing_theorem_trust_gaps_and_role_promotion",
      rationale = "undeclared models, axioms, opaque definitions, and admitted declarations are not proof authority",
    },
    {
      canary_id = "target-and-endianness-drift",
      category = "target",
      execution = "pure",
      expected = "reject",
      test_function = "charon_extraction_contract_rejects_drift_and_weakened_options",
      rationale = "target triple, pointer width, and endianness are proof-relevant inputs",
    },
    {
      canary_id = "source-translation-freshness",
      category = "freshness",
      execution = "pure",
      expected = "reject",
      test_function = "translation_manifest_rejects_stale_links_bad_inventory_and_durable_def_id",
      rationale = "stale translation links, generated files, or durable def_id reuse must fail",
    },
    {
      canary_id = "unsupported-surface",
      category = "unsupported-feature",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_preflight_rejects_unsupported_inputs_and_promotion",
      rationale = "unsupported source must be blocked or failed rather than promoted",
    },
    {
      canary_id = "exported-evidence-tamper",
      category = "freshness",
      execution = "pure",
      expected = "reject",
      test_function = "aeneas_export_tamper_matrix_rejects_digest_role_status_theorem_and_receipt_links",
      rationale = "exported stage, role, status, theorem, and receipt links remain tamper-evident",
    },
  ],
} | Matrix
