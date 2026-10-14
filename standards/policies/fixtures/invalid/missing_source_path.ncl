let contracts = import "../../contracts.ncl" in
let PolicyRegistry = contracts.PolicyRegistry in
{
  schema_version = "fixture",
  checker_ids = ["registry"],
  policies = [
    {
      policy_id = "tigerstyle.fixture.missing-source-path",
      pillar = "fixture",
      source_heading = "Tiger Style Standards",
      source_anchor = "docs/standards/source/octet-standards.adoc#tiger-style-standards",
      summary = "Invalid fixture without source_path.",
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
      fixture_coverage = { status = "planned", compliant = [], noncompliant = [] },
      verification_evidence_requirements = ["must fail"],
      classification_rationale = "Missing source path must fail traceability validation.",
      false_positive_risk = "none",
      existing_enforcement = [],
      custom_checker_justification = "fixture",
    }
  ],
} | PolicyRegistry
