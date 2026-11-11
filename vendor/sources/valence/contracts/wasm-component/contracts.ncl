# Static preflight for bounded Wasm component evidence metadata.
let EmptyLength = 0 in
let Blake3Length = 64 in
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > EmptyLength
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == Blake3Length
) in
let Node = {
  id | NonEmpty,
  role | NonEmpty,
  blake3 | Blake3,
  verification_role | NonEmpty,
  non_claims | Array NonEmpty,
  ..
} in
let Graph = {
  nodes | Array Node,
  edges | Array Dyn,
  ..
} in
{ Node, Graph }
