let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NegativeCase = {
  id | NonEmptyString,
  layer | NonEmptyString,
  mutation | NonEmptyString,
  expected_code | NonEmptyString,
} in
let NegativeCases = {
  schema_version | std.contract.Equal "octet-charon-semantic-negative-cases/v1",
  cases | Array NegativeCase,
} in
{
  schema_version = "octet-charon-semantic-negative-cases/v1",
  cases = [
  { id = "wrong-provider-pin", layer = "profile", mutation = "change the Charon revision", expected_code = "provider-identity" },
  { id = "wrong-provider-profile", layer = "profile", mutation = "change the Aeneas provider profile identity", expected_code = "provider-identity" },
  { id = "broad-selection", layer = "profile", mutation = "allow broad source or item matching", expected_code = "selected-items" },
  { id = "duplicate-selection", layer = "profile", mutation = "repeat a selected item", expected_code = "selected-items" },
  { id = "missing-limit", layer = "profile", mutation = "zero a named execution or graph limit", expected_code = "limits" },
  { id = "stale-reference", layer = "profile", mutation = "change a reference revision or sentinel", expected_code = "reference-boundary" },
  { id = "weakened-non-claims", layer = "profile", mutation = "remove a required non-claim", expected_code = "claim-boundary" },
  { id = "errorful-extraction", layer = "snapshot", mutation = "set translated.has_errors", expected_code = "partial-extraction" },
  { id = "skip-borrowck", layer = "snapshot", mutation = "set options.skip_borrowck", expected_code = "borrow-check" },
  { id = "target-drift", layer = "snapshot", mutation = "change target triple, width, or endianness", expected_code = "target-metadata" },
  { id = "source-drift", layer = "snapshot", mutation = "change embedded selected source bytes", expected_code = "source-identity" },
  { id = "item-drift", layer = "snapshot", mutation = "change the exact local item set", expected_code = "selected-items" },
  { id = "stale-span", layer = "snapshot", mutation = "move an item span outside the bound source", expected_code = "source-span" },
  { id = "missing-call-endpoint", layer = "projection", mutation = "refer to an absent local function ID", expected_code = "graph-endpoint" },
  { id = "duplicate-fact", layer = "projection", mutation = "repeat a semantic node or edge", expected_code = "duplicate-fact" },
  { id = "opaque-item", layer = "projection", mutation = "make a required selected item opaque", expected_code = "unknown-fact" },
  { id = "unsupported-item", layer = "projection", mutation = "remove a required selected item body", expected_code = "unsupported-marker" },
  { id = "call-cycle", layer = "analysis", mutation = "introduce a local call cycle", expected_code = "cycle-observation" },
  { id = "unknown-effect", layer = "analysis", mutation = "introduce an unmapped explicit effect tag", expected_code = "unknown-effect" },
  { id = "comparison-disagreement", layer = "comparison", mutation = "change a source-token fact value", expected_code = "comparison-disagree" },
  { id = "stale-comparison-parent", layer = "comparison", mutation = "change a parent source identity", expected_code = "comparison-unknown" },
  { id = "timeout", layer = "shell", mutation = "exceed the named provider timeout", expected_code = "resource-limit" },
  { id = "log-overflow", layer = "shell", mutation = "exceed a bounded log size", expected_code = "resource-limit" },
  { id = "snapshot-tamper", layer = "receipt", mutation = "change raw snapshot bytes", expected_code = "artifact-identity" },
  { id = "projection-tamper", layer = "receipt", mutation = "change normalized projection bytes", expected_code = "artifact-identity" },
  { id = "reference-promotion", layer = "receipt", mutation = "promote an upstream reference to property evidence", expected_code = "reference-boundary" },
  { id = "formal-proof-promotion", layer = "receipt", mutation = "request formal-proof or property authority", expected_code = "claim-boundary" },
  { id = "missing-provenance", layer = "provenance", mutation = "remove one role-separated artifact provenance record", expected_code = "provenance-coverage" },
  { id = "provenance-tier-promotion", layer = "provenance", mutation = "promote a Charon artifact above transcript assertion", expected_code = "claim-boundary" },
  { id = "default-path-coupling", layer = "cli", mutation = "make cargo octet check require Charon", expected_code = "default-independence" },
  ],
} | NegativeCases
