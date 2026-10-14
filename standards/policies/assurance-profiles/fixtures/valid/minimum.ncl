let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-assurance-profile/v1",
  profile_id = "minimum-valid-profile",
  summary = "Minimal complete profile fixture.",
  scope = {
    repository = "OnixResearch/fixture",
    component = "fixture core",
    source_paths = ["src/lib.rs"],
  },
  assumptions = [{
    assumption_id = "fixture-inputs",
    owner = "fixture",
    statement = "The fixture inputs are exact.",
    validation = "Compare fixture bytes.",
  }],
  trusted_computing_base = [{
    component_id = "fixture-checker",
    owner = "fixture",
    responsibility = "Validate the fixture property.",
    failure_impact = "The fixture result becomes invalid.",
  }],
  mechanisms = [{
    mechanism_id = "fixture-mechanism",
    owner = "fixture",
    assurance_plane = "verification",
    protected_property = "The fixture property is evaluated.",
    assumption_ids = ["fixture-inputs"],
    trusted_base_ids = ["fixture-checker"],
    evidence_ids = ["fixture-evidence"],
    non_claims = ["The fixture does not prove other source."],
  }],
  required_evidence = [{
    evidence_id = "fixture-evidence",
    producer = "fixture",
    artifact_path = "target/fixture.json",
    artifact_kind = "fixture result",
    verification_role = "test",
    verification_scope = "src/lib.rs fixture property",
    validation_command = "cargo test fixture",
    non_claims = ["The result is fixture-scoped."],
  }],
  non_claims = ["The profile is not a universal guarantee."],
} | contracts.AssuranceProfile | contracts.AssuranceProfileRules
