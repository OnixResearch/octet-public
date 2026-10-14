let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Lane = std.contract.from_predicate (fun value => std.array.elem value ["fast", "deep"]) in
let ArtifactKind = std.contract.from_predicate (fun value =>
  std.array.elem value ["wasm", "external-report"]
) in
let Case = {
  case_id | NonEmptyString,
  lane | Lane,
  artifact_kind | ArtifactKind,
  source | NonEmptyString,
  artifact_blake3 | Blake3,
  generator | {
    tool | NonEmptyString,
    version | NonEmptyString,
    seed | NonEmptyString,
    input_identity | NonEmptyString,
  },
  transforms | Array NonEmptyString,
  expected_issue_class | NonEmptyString,
  shrink_predicate | NonEmptyString,
} in
let wat_version = "1.253.0" in
let cohort_version = "0.253.0" in
let wat_case = fun input_case_id input_source input_digest input_expected => {
  case_id = input_case_id,
  lane = "fast",
  artifact_kind = "wasm",
  source = input_source,
  artifact_blake3 = input_digest,
  generator = {
    tool = "wat",
    version = wat_version,
    seed = "not-applicable",
    input_identity = "checked WAT source",
  },
  transforms = ["wat-to-exact-wasm"],
  expected_issue_class = input_expected,
  shrink_predicate = if input_expected == "none" then "not-applicable" else "preserve expected issue class",
} in
{
  schema_version = "octet-spacewasm-corpus/v1",
  profile_id = "spacewasm-mvp",
  source_revision = "e24cf09355a90497148eb5029fdb8e3400bd63e3",
  cases | Array Case = [
    wat_case "positive-mvp-module" "fixtures/generated/mvp-module.wasm" "9855274d998a322359c211238092371419331e92685e4a7858c39508afdd2144" "none",
    wat_case "negative-component" "fixtures/generated/component.wasm" "48d3a2e8a9b0cde6293ba7f68824a97d069d096f44335a9e86aee55238dcdcb8" "disallowed-artifact-kind",
    wat_case "negative-bulk-memory" "fixtures/generated/bulk-memory.wasm" "57a9e00325ec098e2ca8f7b7deb5b9659ccdcbce84d862ef60ea5f16e05e1624" "disallowed-proposal",
    wat_case "negative-sign-extension" "fixtures/generated/sign-extension.wasm" "bd567479f158aa04d550fb4dbcef978f264402a059fa0128cffcf7b0af9baec3" "disallowed-proposal",
    wat_case "negative-nontrapping-conversion" "fixtures/generated/nontrapping-conversion.wasm" "4b5134e8b092279e226c28a79f675c1586d6a044ae7aefd10cace29e3afebb6b" "disallowed-proposal",
    wat_case "negative-wrong-import" "fixtures/generated/wrong-import.wasm" "0bc7a928c4349245d0f7d6b21c326203446d642ecf52c4d95ce486c0a558edd7" "disallowed-import",
    wat_case "negative-wrong-abi" "fixtures/generated/wrong-abi.wasm" "3e91917b57a81f718565f4ce97e5566d64a1e788c5ddbbf7a8014218dfc2cb13" "abi-mismatch",
    wat_case "negative-memory-limit" "fixtures/generated/memory-limit.wasm" "767b9b6f8451c27570f5f528eb0cf7411d1155e9e1265b9bed2d54add8c1851b" "memory-limit",
    wat_case "negative-table-limit" "fixtures/generated/table-limit.wasm" "d3ab96cfb77acab121c8a5f20821e875915ca973d9a42c437da394c0dc05af75" "table-limit",
    wat_case "negative-parameter-limit" "fixtures/generated/parameter-limit.wasm" "9a9ff231b09e89e145853b7e6a11e0752e9a6bfab19d18a8edc4d090a7e4b59c" "structural-limit",
    wat_case "negative-memory-grow" "fixtures/generated/memory-grow.wasm" "151e3873d0d47044ec99c3de67beb1825d63c5e0e8f085da9192ea024918ec7e" "memory-growth",
    wat_case "negative-start-function" "fixtures/generated/start-function.wasm" "3ebf05ed7f53e6ef8ca13f3b3d09512f2edefdf137003011d5249c1e870cefa7" "start-function",
    {
      case_id = "generated-smith-negative",
      lane = "deep",
      artifact_kind = "wasm",
      source = "fixtures/generated/smith-fast.wasm",
      artifact_blake3 = "f456df26295a041513d62ccfb0b52ac694ddadcff902101e9c36169a0724728d",
      generator = {
        tool = "wasm-smith",
        version = cohort_version,
        seed = "fixtures/seeds/smith-fast.seed",
        input_identity = "b3:c494fbef068b46d1a1b91f7ee28fe9cac974a8938aad73724d7aee86e1c1995e",
      },
      transforms = ["deterministic decision-tree generation"],
      expected_issue_class = "disallowed-proposal",
      shrink_predicate = "preserve at least one spacewasm-mvp rejection",
    },
    {
      case_id = "mutated-mvp-module",
      lane = "deep",
      artifact_kind = "wasm",
      source = "fixtures/generated/mutated-mvp.wasm",
      artifact_blake3 = "ca384ee74699328779c1057f16223ccef8173b3f31160b237172dba2e6adfa70",
      generator = {
        tool = "wasm-mutate",
        version = cohort_version,
        seed = "1701",
        input_identity = "b3:9855274d998a322359c211238092371419331e92685e4a7858c39508afdd2144",
      },
      transforms = ["pseudo-random deterministic mutation"],
      expected_issue_class = "none",
      shrink_predicate = "preserve spacewasm-mvp conformance",
    },
    {
      case_id = "external-recorded-only",
      lane = "fast",
      artifact_kind = "external-report",
      source = "fixtures/external/chaoscontrol-recorded.json",
      artifact_blake3 = "b596ea24bd0d025eae0e4440cdb9c244d5be1153a5d219009e1aee2c55a0122a",
      generator = {
        tool = "fixture",
        version = "octet-spacewasm-external-runtime-report/v1",
        seed = "not-applicable",
        input_identity = "b3:75bb5e89bbf581910256a09de594fe701c948ed4d599184db2cd543ee8387351",
      },
      transforms = [],
      expected_issue_class = "none",
      shrink_predicate = "not-applicable",
    },
    {
      case_id = "external-overclaim",
      lane = "fast",
      artifact_kind = "external-report",
      source = "fixtures/external/overclaim.json",
      artifact_blake3 = "4dc03298c3ecccf782a18c1d34b8f0a1cec48d35b0e390cd46c40c579c7fa60a",
      generator = {
        tool = "fixture",
        version = "octet-spacewasm-external-runtime-report/v1",
        seed = "not-applicable",
        input_identity = "b3:0058a9a3f2e036eb885c232821a178817baa3becf1ee06803ef393088a02b46d",
      },
      transforms = ["promote verification role beyond recorded-only"],
      expected_issue_class = "external-evidence-mismatch",
      shrink_predicate = "preserve overclaim rejection",
    },
  ],
}
