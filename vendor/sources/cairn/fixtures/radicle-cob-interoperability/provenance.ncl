let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let GitOid = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let MaxTestSeedByte = 255 in
let TestSeedByte = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0 && value <= MaxTestSeedByte
) in
let Provenance = {
  schema | NonEmptyString,
  producer | {
    implementation | NonEmptyString,
    radicle_version | NonEmptyString,
    radicle_cob_version | NonEmptyString,
    radicle_crypto_version | NonEmptyString,
    radicle_oid_version | NonEmptyString,
    git2_version | NonEmptyString,
    generation_command | NonEmptyString,
    bundle_command | NonEmptyString,
  },
  fixture | {
    bundle_path | NonEmptyString,
    bundle_blake3 | Blake3Hex,
    producer_source_path | NonEmptyString,
    producer_source_blake3 | Blake3Hex,
    producer_manifest_path | NonEmptyString,
    producer_manifest_blake3 | Blake3Hex,
    producer_lock_path | NonEmptyString,
    producer_lock_blake3 | Blake3Hex,
    actor | NonEmptyString,
    delegate | NonEmptyString,
    delegate_seed_byte | TestSeedByte,
    base_oid | GitOid,
    object_id | GitOid,
    operation_id | GitOid,
    reference | NonEmptyString,
    change_id | NonEmptyString,
    repository_id | NonEmptyString,
    type_name | NonEmptyString,
    proposal | NonEmptyString,
  },
  non_claims | Array NonEmptyString,
} in
({
  schema = "cairn.radicle-cob-interoperability-provenance.v1",
  producer = {
    implementation = "radicle::storage::git::Repository standalone fixture producer",
    radicle_version = "0.24.0",
    radicle_cob_version = "0.20.0",
    radicle_crypto_version = "0.17.0",
    radicle_oid_version = "0.2.0",
    git2_version = "0.20.4",
    generation_command = "cargo run --locked --manifest-path fixtures/radicle-cob-interoperability/producer/Cargo.toml -- /tmp/cairn-cob-reference-repository",
    bundle_command = "git -C /tmp/cairn-cob-reference-repository bundle create fixtures/radicle-cob-interoperability/external-cairn-cob.bundle --all",
  },
  fixture = {
    bundle_path = "fixtures/radicle-cob-interoperability/external-cairn-cob.bundle",
    bundle_blake3 = "8274f326f3165f7b19ee69aaa0dcd7a9db02c75fc0c63ed3609150676ab3396d",
    producer_source_path = "fixtures/radicle-cob-interoperability/producer/src/main.rs",
    producer_source_blake3 = "c811885612fe90a4b3b896baa70d62082637f75ce51c34d87756050a5d674b51",
    producer_manifest_path = "fixtures/radicle-cob-interoperability/producer/Cargo.toml",
    producer_manifest_blake3 = "f806d96286a7e0c38ab91eeba9f13d59ba158abb5d6b80063070bf0bd7cc249e",
    producer_lock_path = "fixtures/radicle-cob-interoperability/producer/Cargo.lock",
    producer_lock_blake3 = "e77035dd3e1fa6c13f6d1a198f7f9a75068353636cd60b8e3eccf133c49fca5c",
    actor = "z6MkwJFpvJVFe15ygGgYLTYuiPwfhkkDRkMG4eKQFT8g65yE",
    delegate = "z6MkgAnvkP45uNxwCKeNdt6wrYkEjpYX4f7Nrd8MQqFL8Fbn",
    delegate_seed_byte = 42,
    base_oid = "e8de0411831a4e23f41ee9853c9f75bc4b77daaf",
    object_id = "dfc1809018f6941bf514d15d644b2b8993ab15d2",
    operation_id = "dfc1809018f6941bf514d15d644b2b8993ab15d2",
    reference = "refs/namespaces/z6MkwJFpvJVFe15ygGgYLTYuiPwfhkkDRkMG4eKQFT8g65yE/refs/cobs/xyz.onixresearch.cairn-change/dfc1809018f6941bf514d15d644b2b8993ab15d2",
    change_id = "external-cob-fixture",
    repository_id = "workspace:external-cob-fixture",
    type_name = "xyz.onixresearch.cairn-change",
    proposal = "# External reference proposal\n",
  },
  non_claims = [
    "the fixture proves compatibility only for the pinned producer and format-library cohort",
    "the fixture does not prove compatibility with arbitrary Radicle versions or COB types",
    "the fixture does not prove publication, synchronization, convergence, remote trust, durability, or release eligibility",
    "deterministic fixture keys are public test values and grant no production authority",
  ],
}) | Provenance
