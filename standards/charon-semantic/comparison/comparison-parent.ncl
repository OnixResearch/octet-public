let NonEmptyString = std.contract.from_predicate (fun value => std.string.is_match "^.+$" value) in
let B3Identity = std.contract.from_predicate (fun value => std.string.is_match "^b3:[0-9a-f]{64}$" value) in
let Fact = {
  item | NonEmptyString,
  fact | NonEmptyString,
  value | NonEmptyString,
} in
let ComparisonParent = {
  schema_version | std.contract.Equal "octet-charon-semantic-comparison-parent/v1",
  authority | std.contract.Equal "source-token",
  identity | B3Identity,
  source_identity | B3Identity,
  facts | Array Fact,
} in
{
  schema_version = "octet-charon-semantic-comparison-parent/v1",
  authority = "source-token",
  identity = "b3:23e41fc9fa5d85379bac551522134416b0478cbaf19d67d35e69a6f0b53b906e",
  source_identity = "b3:fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
  facts = [
    { item = "real_fixture::safe_add", fact = "item-presence", value = "present" },
    { item = "real_fixture::safe_add", fact = "control-flow", value = "structured" },
    { item = "real_fixture::safe_add", fact = "effect-candidate", value = "panic" },
  ],
} | ComparisonParent
