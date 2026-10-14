let contracts = import "../../contracts.ncl" in
{
  schema_version = "fixture",
  checker_ids = ["octet-standards"],
  policies = [
    {
      policy_id = "tigerstyle.fixture.capability-manifest-empty-core",
      pillar = "architecture",
      source_path = "docs/standards/source/architecture.adoc",
      source_heading = "Effect capability boundaries",
      source_anchor = "docs/standards/source/architecture.adoc#effect-capability-boundaries",
      summary = "Invalid strict capability manifest fixture.",
      classification = "hard",
      enforcement_level = "hard",
      current_enforcement_mode = "structural",
      target_enforcement_mode = "structural",
      escalation_criteria = "strict manifests must declare core scopes",
      mechanisms = [{
        kind = "capability-api",
        description = "strict capability manifest with no core scopes",
        exported_obligations = ["declared core scopes reject direct platform effects"],
        capability_enforcement_mode = "strict",
        capability_scope_classes = ["core"],
        capability_families = ["time"],
      }],
      checker_owner = "architecture",
      checker_id = "octet-standards",
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
      verification_evidence_requirements = ["invalid capability manifest fixture"],
      classification_rationale = "Fixture proves strict capability manifests fail without core scope declarations.",
      false_positive_risk = "none",
      existing_enforcement = ["octet-standards"],
      custom_checker_justification = "fixture",
    }
  ],
} | contracts.PolicyRegistry | contracts.PolicyRegistryRules
