# Typed starter profile registry for optional dynamic evidence rails.
# Export with: nickel export --format json templates/dynamic-evidence-profiles.ncl

let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Timing = std.contract.from_predicate (fun value => std.array.elem value [
  "after-big-change",
  "focused-ci",
  "nightly",
  "pre-release",
  "suspected-bug",
]) in
let DynamicEvidenceProfile = {
  name | NonEmptyString,
  risk_class | NonEmptyString,
  tool | NonEmptyString,
  timing | Timing,
  allowed_commands | Array NonEmptyString,
  expected_signal | NonEmptyString,
  not_default | Bool | default = true,
  non_claim | NonEmptyString,
} in
{
  schema_version = "octet-dynamic-evidence-profiles/v1",
  profiles = [
    {
      name = "miri-unsafe-boundary",
      risk_class = "unsafe-boundary undefined-behavior smoke",
      tool = "cargo miri",
      timing = "pre-release",
      allowed_commands = ["cargo miri test"],
      expected_signal = "passed",
      not_default = true,
      non_claim = "Miri evidence covers the selected interpreter workload only and is not a whole-program correctness proof.",
    },
    {
      name = "loom-concurrency",
      risk_class = "bounded concurrency schedule exploration",
      tool = "cargo test with loom cfg",
      timing = "suspected-bug",
      allowed_commands = ["cargo test --cfg loom"],
      expected_signal = "passed",
      not_default = true,
      non_claim = "Loom evidence covers the selected model and bounds only and is not exhaustive runtime verification.",
    },
  ],
} | {
  schema_version | std.contract.from_predicate (fun value => value == "octet-dynamic-evidence-profiles/v1"),
  profiles | Array DynamicEvidenceProfile,
}
