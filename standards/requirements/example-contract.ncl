# Reviewed requirement-to-evidence contract fixture.
#
# Human-authored policy stays in Nickel. Export deterministic JSON with:
#   nickel export --format json standards/requirements/example-contract.ncl \
#     > standards/requirements/example-contract.json
{
  schema_version = "octet-requirement-contract/v1",
  contract_id = "octet.requirements.example",
  reviewed_specification = {
    path = "standards/requirements/example-spec.md",
    blake3 = "78afa0f6c7761fbbf7f727507508410660866dd75ff35618ea27209478cc0814",
  },
  clauses = [
    {
      clause_id = "octet.aeneas_lean_proof_rail.theorem_adequacy",
      statement = "The checked Aeneas/Lean fixture theorem aeneas.fixture.safe_add is present and accepted over the generated model.",
      required = true,
      disposition = "formal_proof",
      checked_scope = [
        "formal-proof:fixture-only:octet.aeneas_lean_proof_rails.fixture#rail-0",
      ],
      expected_evidence_tier = "formal_proof",
      allowed_evidence_ids = ["aeneas.fixture.safe_add"],
      allowed_source_requirement_ids = [
        "octet.aeneas_lean_proof_rail.theorem_adequacy",
      ],
      required_evidence_classes = ["proof_card"],
      required_negative_controls = [],
      required_provider_id = "aeneas-lean",
      required_bindings = {
        source_ids = [
          "formal-proof:fixture-only:octet.aeneas_lean_proof_rails.fixture#rail-0",
        ],
        policy_ids = ["octet.aeneas_lean_proof_rails.fixture"],
        tool_identity_hashes = [
          "4ff9ff39f3ec4e1015d318f317230c2a568a3aded1b49c07a9ab814af2cdbefa",
        ],
        artifact_hashes = [
          "1328a8e93950d7cf39724b33b7a932f6c544ea881dd6c44a0e31c8cf0aa3d8b5",
        ],
        checked_scope = [],
        evidence_tier = "formal_proof",
      },
      source_bindings = [
        {
          locator = "standards/proof-rails/aeneas-lean/fixtures/valid/real_fixture/src/lib.rs",
          blake3 = "fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
        },
        {
          locator = "standards/proof-rails/aeneas-lean/fixtures/valid/real_fixture/Theorem.lean",
          blake3 = "08e7c2d1fb1e21b2e7a174340a9f5b87c3acc634c591c6102624cd3a231f1125",
        },
      ],
    },
    {
      clause_id = "octet.requirements.example.assumption.lean_toolchain",
      statement = "The fixture is evaluated against the pinned Lean toolchain bytes.",
      required = true,
      disposition = "accepted_assumption",
      checked_scope = ["fixture:lean-toolchain"],
      expected_evidence_tier = "empirical_check",
      allowed_evidence_ids = ["check.lean_toolchain"],
      required_evidence_classes = ["empirical_check"],
      assumption_id = "assumption.lean_toolchain",
      required_bindings = {
        source_ids = [
          "standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/lean-toolchain",
        ],
        policy_ids = [],
        tool_identity_hashes = [],
        artifact_hashes = [],
        checked_scope = [],
        evidence_tier = "empirical_check",
      },
      source_bindings = [
        {
          locator = "standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/lean-toolchain",
          blake3 = "4ff9ff39f3ec4e1015d318f317230c2a568a3aded1b49c07a9ab814af2cdbefa",
        },
      ],
    },
    {
      clause_id = "octet.requirements.example.excluded.gpu_paths",
      statement = "GPU execution paths are outside this fixture's reviewed scope.",
      required = false,
      disposition = "excluded",
      exclusion_reason = "The fixture covers only the checked Aeneas/Lean CPU proof receipt.",
    },
  ],
  assumptions = [
    {
      assumption_id = "assumption.lean_toolchain",
      statement = "The retained Lean toolchain bytes are the reviewed toolchain identity.",
      identity = "lean-toolchain:4ff9ff39f3ec4e1015d318f317230c2a568a3aded1b49c07a9ab814af2cdbefa",
      trusted_boundary_identity = "aeneas-lean-provider-pin:bc2eb6f941369fa3c695e30598b5758b11e4f0de",
      affected_clause_ids = [
        "octet.requirements.example.assumption.lean_toolchain",
      ],
      required_empirical_evidence = ["check.lean_toolchain"],
    },
  ],
  exclusions = [
    {
      exclusion_id = "exclusion.gpu_paths",
      clause_id = "octet.requirements.example.excluded.gpu_paths",
      reason = "The fixture covers only the checked Aeneas/Lean CPU proof receipt.",
      reviewed_specification_blake3 = "78afa0f6c7761fbbf7f727507508410660866dd75ff35618ea27209478cc0814",
    },
  ],
  non_claims = [
    "does not authenticate the reviewer or producer",
    "does not prove Aeneas, Charon, or Lean soundness",
    "does not prove whole-program correctness",
    "does not establish release eligibility",
  ],
}
