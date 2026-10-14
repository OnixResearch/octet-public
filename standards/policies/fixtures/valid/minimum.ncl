let contracts = import "../../contracts.ncl" in
let PolicyRegistry = contracts.PolicyRegistry in
{
  schema_version = "fixture",
  checker_ids = ["registry"],
  policies = [
    {
      policy_id = "tigerstyle.fixture.valid",
      pillar = "fixture",
      source_path = "docs/standards/source/octet-standards.adoc",
      source_heading = "Tiger Style Standards",
      source_anchor = "docs/standards/source/octet-standards.adoc#tiger-style-standards",
      summary = "Valid fixture policy entry.",
      classification = "hard",
      enforcement_level = "hard",
      checker_owner = "fixture",
      checker_id = "registry",
      default_severity = "error",
      suppression_requirements = {
        requires_policy_id = true,
        requires_source_section = true,
        requires_owner = true,
        requires_rationale = true,
        requires_review_trigger = true,
        requires_safer_default = true,
      },
      fixture_coverage = {
        status = "planned",
        compliant = [],
        noncompliant = [],
      },
      verification_evidence_requirements = ["validates contract"],
      classification_rationale = "Fixture proves the contract accepts complete entries.",
      false_positive_risk = "none",
      existing_enforcement = ["registry"],
      custom_checker_justification = "fixture",
    }
  ],
} | PolicyRegistry
