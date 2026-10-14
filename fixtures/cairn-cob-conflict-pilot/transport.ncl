let Actor = { actor | String, ref_name | String, resolved_target | String } in
let Transport = {
  cairn_commit | String, synthetic_base_git_oid | String, object_id | String,
  creator | Actor, delegate | Actor,
  resolution | { direct_parents | Array String, losing_revision_reachable | Bool },
  source_remotes | Array String, convergence_remotes | Array String,
  reload_remotes | Array String, valid | Bool, non_claims | Array String,
} in
{
  cairn_commit = "6b7e19531554714b6760e8cacc2a980e576d36af",
  synthetic_base_git_oid = "b91fdcff9139492e55a7e64a142de6d0684ddb03",
  object_id = "030a0e27d379e6e5ff67f6eac940e2369a8169b1",
  creator = {
    actor = "z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5",
    ref_name = "refs/namespaces/z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    resolved_target = "61922203bead59f454200971ffcd9e016f84840e",
  },
  delegate = {
    actor = "z6MkvHHbrGisC4BxN9M8b95EdXTidoFjeRE3i7Qc355d6yqU",
    ref_name = "refs/namespaces/z6MkvHHbrGisC4BxN9M8b95EdXTidoFjeRE3i7Qc355d6yqU/refs/cobs/xyz.onixresearch.cairn-change/030a0e27d379e6e5ff67f6eac940e2369a8169b1",
    resolved_target = "91ad968acd64ce5d4d26439e9ba72261c96e2653",
  },
  resolution = {
    direct_parents = ["91ad968acd64ce5d4d26439e9ba72261c96e2653", "bdac0c8ff7cc99c781d382d43e69b9a8e10dc1fe"],
    losing_revision_reachable = true,
  },
  source_remotes = [],
  convergence_remotes = [],
  reload_remotes = [],
  valid = true,
  non_claims = ["shell-measured Git observations, not a Cairn-emitted receipt or Radicle runtime observation", "local Git bundle transport and reload observations do not prove semantic merge correctness, synchronization, distributed convergence, durability, or release eligibility"],
} | Transport
