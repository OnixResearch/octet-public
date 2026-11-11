# Closed contracts for the bounded storage-fault assurance profile.
#
# The profile links exact saved producer and Cairn receipt identities. Octet
# validates the saved files only; it never starts the producer.
let NonEmptyString =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.length value > 0
  )
in
let NonEmptyArray =
  std.contract.from_predicate (fun value =>
    std.is_array value && std.array.length value > 0
  )
in
let OneOf = fun allowed =>
  std.contract.from_predicate (fun value =>
    std.is_string value && std.array.elem value allowed
  )
in
let Blake3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in
let ProfileSchema = OneOf ["octet-storage-fault-assurance/v1"] in
let CaseKind = OneOf ["positive", "negative"] in
let RequiredCaseContract = {
  case_id | NonEmptyString,
  kind | CaseKind,
} in
let ReceiptBindingContract = {
  path | NonEmptyString,
  blake3 | Blake3Identity,
} in
let has_duplicates = fun values =>
  std.array.length values != std.array.length (std.array.dedup values)
in
let ProfileRulesContract = std.contract.from_predicate (fun profile =>
  !has_duplicates (profile.required_cases |> std.array.map (fun case => case.case_id))
  && std.array.any (fun case => case.kind == "negative") profile.required_cases
) in
{
  CaseKind = CaseKind,
  RequiredCase = RequiredCaseContract,
  ReceiptBinding = ReceiptBindingContract,
  Profile = {
    schema_version | ProfileSchema,
    profile_id | NonEmptyString,
    source_blake3 | Blake3Identity,
    candidate_blake3 | Blake3Identity,
    required_cases | NonEmptyArray | Array RequiredCaseContract,
    producer_receipt | ReceiptBindingContract,
    cairn_receipt | ReceiptBindingContract,
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
  ProfileRules = ProfileRulesContract,
}
