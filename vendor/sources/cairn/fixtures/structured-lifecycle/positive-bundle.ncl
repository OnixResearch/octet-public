let Contracts = import "../../cairn-policy/structured-lifecycle-contracts.ncl" in
({
  schema = "cairn.structured-lifecycle.bundle.v1",
  format_metadata = {
    source_format = "nickel",
  },
  specs = [{
    id = "structured-lifecycle",
    purpose = "Fixture for typed Cairn lifecycle source.",
    requirements = [{
      id = "cairn.structured.fixture.valid",
      strength = "MUST",
      statement = "validate structured lifecycle source",
      scenarios = [{
        id = "valid_bundle",
        given = ["a typed Nickel lifecycle bundle"],
        "when" = ["Cairn exports it to canonical runtime data"],
        "then" = ["the generated bundle validates"],
      }],
    }],
  }],
  changes = [{
    id = "demo-change",
    tasks = [{
      id = "validate-fixture",
      covers = ["cairn.structured.fixture.valid"],
    }],
    probes = [{
      id = "validate-fixture-probe",
      requirements = ["cairn.structured.fixture.valid"],
      coverage_class = "positive",
      evidence_role = "probe",
    }],
  }],
}) | Contracts.Bundle
