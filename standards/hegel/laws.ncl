let Law = {
  law_id | String,
  pure_core | String,
  generator | String,
  positive_property | String,
  negative_property | String,
  generated_input_benefit | String,
  case_budget | Number,
  replay_method | String,
  fixture_complement | String,
  effect_free | Bool,
}
in
{
  schema_version = "octet-hegel-law-catalog/v1",
  laws = [
    {
      law_id = "octet.hegel.issue-code-normalization-idempotent",
      pure_core = "cargo_octet::hegel::normalize_issue_codes",
      generator = "bounded vectors of bounded Unicode strings",
      positive_property = "normalizing twice equals normalizing once and produces sorted unique non-empty values",
      negative_property = "raw generated input is not assumed to equal its normalized projection",
      generated_input_benefit = "generated duplicates, whitespace, ordering, and Unicode combinations exceed the concrete fixture matrix",
      case_budget = 64,
      replay_method = "Hegel derandomized replay plus retained counterexample issue codes",
      fixture_complement = "standards/evidence/fixtures/valid/hegel_hardened_details_v1.ncl",
      effect_free = true,
    } | Law,
  ],
}
