let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Natural = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0
) in
let Case = {
  case_id | NonEmptyString,
  lane | std.contract.from_predicate (fun value => std.array.elem value ["fast", "deep"]),
  source | NonEmptyString,
  generator | {
    tool | NonEmptyString,
    version | NonEmptyString,
    seed | Natural,
  },
  transforms | Array NonEmptyString,
  expected_issue_class | NonEmptyString,
  shrink_predicate | NonEmptyString,
} in
let fast_seed = 1701 in
let deep_seed = 9001 in
{
  schema_version = "octet-wasm-artifact-corpus/v1",
  cases | Array Case = [
    {
      case_id = "positive-core-module",
      lane = "fast",
      source = "positive/module.wat",
      generator = { tool = "wat", version = "1.253.0", seed = fast_seed },
      transforms = [],
      expected_issue_class = "none",
      shrink_predicate = "not-applicable",
    },
    {
      case_id = "positive-empty-component",
      lane = "fast",
      source = "positive/component.wat",
      generator = { tool = "wit-smith", version = "0.253.0", seed = fast_seed },
      transforms = ["materialize-equivalent-empty-component-wat"],
      expected_issue_class = "none",
      shrink_predicate = "not-applicable",
    },
    {
      case_id = "negative-undeclared-wasi",
      lane = "fast",
      source = "negative/undeclared-wasi.wat",
      generator = { tool = "wasm-smith", version = "0.253.0", seed = fast_seed },
      transforms = ["inject-undeclared-wasi-import"],
      expected_issue_class = "disallowed-import",
      shrink_predicate = "preserve disallowed-import",
    },
    {
      case_id = "negative-sign-extension-proposal",
      lane = "fast",
      source = "negative/sign-extension.wat",
      generator = { tool = "wasm-smith", version = "0.253.0", seed = fast_seed },
      transforms = ["enable-sign-extension-op"],
      expected_issue_class = "disallowed-proposal",
      shrink_predicate = "preserve disallowed-proposal",
    },
    {
      case_id = "negative-memory-growth",
      lane = "fast",
      source = "negative/memory-grow.wat",
      generator = { tool = "wasm-mutate", version = "0.253.0", seed = fast_seed },
      transforms = ["insert-memory-grow"],
      expected_issue_class = "memory-growth",
      shrink_predicate = "preserve memory-growth",
    },
    {
      case_id = "negative-start-function",
      lane = "deep",
      source = "negative/start-function.wat",
      generator = { tool = "wasm-shrink", version = "0.253.0", seed = deep_seed },
      transforms = ["retain-start-section"],
      expected_issue_class = "start-function",
      shrink_predicate = "preserve start-function",
    },
  ],
}
