let Receipt = { name | String, blake3_hex | String } in
let ActorHead = {
  actor | String, ref_name | String, operation_id | String,
  content_blake3 | String, content_bytes | Number, content_root | String,
  state_hash | String, retained_path | String,
} in
let BundleHead = { ref_name | String, target | String } in
let Bundle = { path | String, blake3_hex | String, heads | Array BundleHead } in
let Resolution = {
  operation_id | String, selected_revision | String, losing_revision | String,
  direct_parents | Array String, content_root | String, state_hash | String,
} in
let ConflictPilotManifest = {
  schema | String, change | String, workspace_id | String, backend | String,
  cairn_commit | String, synthetic_base_git_oid | String, policy_hash | String,
  object_id | String, root_operation_id | String, root_content_root | String,
  root_state_hash | String, creator | ActorHead, delegate | ActorHead,
  conflict_state_hash | String, resolution | Resolution,
  resolved_lifecycle_condition | String, bundle | Bundle,
  receipts | Array Receipt, required_non_claim_fragments | Array String,
} in
{
  schema = "octet.cairn-cob-conflict-pilot.v2",
  change = "exercise-cairn-cob-conflict-recovery",
  workspace_id = "workspace:synthetic-cob-conflict",
  backend = "radicle-cob.v1",
  cairn_commit = "6b7e19531554714b6760e8cacc2a980e576d36af",
  synthetic_base_git_oid = "b91fdcff9139492e55a7e64a142de6d0684ddb03",
  policy_hash = "c8276321e0e82c524e09ad9357fa48a9d38ac8590263ee6f79c51a328dc6037d",
  object_id = "030a0e27d379e6e5ff67f6eac940e2369a8169b1",
  root_operation_id = "030a0e27d379e6e5ff67f6eac940e2369a8169b1",
  root_content_root = "7c4dedd4299257992f0ac8dad67237b86f3b0cfd262568626ca866540b3291ef",
  root_state_hash = "0ab228f39e86943aedfd940c001a24b76462e69f30832aaa031340e84cb06d3c",
  creator = {
    actor = "z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5",
    ref_name = "refs/namespaces/z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    operation_id = "bdac0c8ff7cc99c781d382d43e69b9a8e10dc1fe",
    content_blake3 = "8f3e024e6456bd60b64831bac8808eab13f9006341eb5ffcb00d3a787fe7827c",
    content_bytes = 60,
    content_root = "45b5026f17e748f810599399b88556c01df8fec4311bf5dc4692e6691d0b827f",
    state_hash = "0eb341a47e974b2e3b00d06b4101fecd2ed0a816b691ff308d545a338a6ec28e",
    retained_path = "heads/creator-tasks.md",
  },
  delegate = {
    actor = "z6MkvHHbrGisC4BxN9M8b95EdXTidoFjeRE3i7Qc355d6yqU",
    ref_name = "refs/namespaces/z6MkvHHbrGisC4BxN9M8b95EdXTidoFjeRE3i7Qc355d6yqU/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    operation_id = "91ad968acd64ce5d4d26439e9ba72261c96e2653",
    content_blake3 = "bf05be03e78208d0e3a9d8a2b97788934291b6083dd6021f993b338acd49b1fc",
    content_bytes = 61,
    content_root = "a4f2155332f70d6d1bbdef07714e553f2fc9bf0d423ce0f23201e0bf80e8d584",
    state_hash = "d0090be3c37380633c0089e641572926c580253f728d3a7d8ce9b3f722008ba6",
    retained_path = "heads/delegate-tasks.md",
  },
  conflict_state_hash = "f509b594ec400ce0c6c6be7c84cd90df2c9eb4ae6ba62f0f0fd526dd21e8bfb3",
  resolution = {
    operation_id = "61922203bead59f454200971ffcd9e016f84840e",
    selected_revision = "bdac0c8ff7cc99c781d382d43e69b9a8e10dc1fe",
    losing_revision = "91ad968acd64ce5d4d26439e9ba72261c96e2653",
    direct_parents = ["91ad968acd64ce5d4d26439e9ba72261c96e2653", "bdac0c8ff7cc99c781d382d43e69b9a8e10dc1fe"],
    content_root = "45b5026f17e748f810599399b88556c01df8fec4311bf5dc4692e6691d0b827f",
    state_hash = "675181463713e9277193a3c9f081b5276992a31234ffc353a7d0187e8858279e",
  },
  resolved_lifecycle_condition = "stale",
  bundle = {
    path = "octet-cob-conflict.bundle",
    blake3_hex = "8c8475bc5745607104365c3b52dab656061595264ede305178a8848c95f937ef",
    heads = [{
      target = "b91fdcff9139492e55a7e64a142de6d0684ddb03",
      ref_name = "refs/heads/main",
    }, {
      target = "61922203bead59f454200971ffcd9e016f84840e",
      ref_name = "refs/namespaces/z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    }, {
      target = "91ad968acd64ce5d4d26439e9ba72261c96e2653",
      ref_name = "refs/namespaces/z6MkvHHbrGisC4BxN9M8b95EdXTidoFjeRE3i7Qc355d6yqU/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    }],
  },
  receipts = [{
    name = "checkout-third-execute.json",
    blake3_hex = "2d5f6df53d53269f558f30a1afa3dd93b883d1642312e15fa2127c6604077615",
  }, {
    name = "checkout-third-preview.json",
    blake3_hex = "68cf2bc6e7423ec34147a46fbfd9be0efaabeebeb5ffeea7d3854cb8152f975d",
  }, {
    name = "conflicts-metadata.json",
    blake3_hex = "567504f52696eb800a6bee958c40b92c18dec57944c08d9a3153d62dd53b9ea6",
  }, {
    name = "conflicts-resolved-third.json",
    blake3_hex = "9f66db2d42b12e382c9c0938dbd0b651e4b5e204b8a79b94513bcdb0a982f443",
  }, {
    name = "conflicts-show-creator.json",
    blake3_hex = "466420425401dce0f77615926d6cfea3597a2577e50a1e38085bf078bf659c98",
  }, {
    name = "conflicts-show-delegate.json",
    blake3_hex = "19fe46096bcbd0ed3c9906dddda34a87d69316c38226399eb7064fedb8fd4875",
  }, {
    name = "creator-update-execute.json",
    blake3_hex = "b3b20ec48fb086ae6f35dc32d3ba3192f3c0ed65352b2e60b6a26218b4c8f24e",
  }, {
    name = "creator-update-preview.json",
    blake3_hex = "00cf383639199d89adadf7bf7600ae886d958f653848b92f54bb5920218a7051",
  }, {
    name = "delegate-update-execute.json",
    blake3_hex = "c44822ab81d5ddf3763bb2289ca4faa755bf6661088b8c8dd64ab7619ad09e1b",
  }, {
    name = "delegate-update-preview.json",
    blake3_hex = "24fb99864d2127d6ec7996d546b07802f2738e3ae48eac586916b6d5d85052ef",
  }, {
    name = "import-execute.json",
    blake3_hex = "7be307843c603bd9ece9280b55492bff15da7007c339716dc745134ff7268ba7",
  }, {
    name = "import-preview.json",
    blake3_hex = "8fe31d5ad14685c648eb2ac3d5dfded7fa45202cb459c6f114ef75c70d5600e9",
  }, {
    name = "resolve-execute.json",
    blake3_hex = "e24a8538d16b36e36a9f7a21f9553e141e2267c8b3278340a7bed27d99082709",
  }, {
    name = "resolve-preview.json",
    blake3_hex = "8323a3d4aad2ec3b3905ad01ab27f0bd997ba363dd97a8b9f1593e85a38eaf0a",
  }, {
    name = "status-third-after-checkout.json",
    blake3_hex = "7801dce523e3ad227e465eb0416029f2c5a125d61570e8eab4da78b5eeba83d0",
  }, {
    name = "status-third-before-checkout.json",
    blake3_hex = "91307ec0f72fdb646a3e605516c9aefb7b403a5e4ef0ef71db99b262fb7ab76e",
  }, {
    name = "transport.json",
    blake3_hex = "74259edf4d455867e186a50f913b34c175cc461082151e4f496966c57f24ae0d",
  }, {
    name = "update-conflict-negative.json",
    blake3_hex = "6ba81a482fe7583a367ae9e5c861b51d270f69127b658978ed4ecfcfd631ac21",
  }],
  required_non_claim_fragments = ["semantic merge correctness", "synchronization", "distributed convergence", "durability", "release eligibility"],
} | ConflictPilotManifest
