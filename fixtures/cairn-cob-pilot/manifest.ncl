let Receipt = {
  name | String,
  blake3_hex | String,
} in
let Transport = {
  source_ref_target | String,
  destination_ref_target | String,
  source_remotes | Array String,
  destination_remotes | Array String,
  artifact_tree_blake3 | String,
} in
let BundleHead = {
  ref_name | String,
  target | String,
} in
let Bundle = {
  path | String,
  blake3_hex | String,
  heads | Array BundleHead,
} in
let PilotManifest = {
  schema | String,
  change | String,
  workspace_id | String,
  backend | String,
  cairn_commit | String,
  base_git_oid | String,
  actor | String,
  object_id | String,
  import_operation_id | String,
  update_operation_id | String,
  final_content_root | String,
  final_state_hash | String,
  namespace_ref | String,
  bundle | Bundle,
  transport | Transport,
  receipts | Array Receipt,
  required_non_claim_fragments | Array String,
} in
{
  schema = "octet.cairn-cob-pilot.v1",
  change = "adopt-cairn-cob-collaboration",
  workspace_id = "workspace:octet",
  backend = "radicle-cob.v1",
  cairn_commit = "6b7e19531554714b6760e8cacc2a980e576d36af",
  base_git_oid = "51bd1ec2dd218f068d1d55d01781e38c877dd063",
  actor = "z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5",
  object_id = "5be37f1c60fcebd2db91101c38203b18eb32f877",
  import_operation_id = "5be37f1c60fcebd2db91101c38203b18eb32f877",
  update_operation_id = "649fa8cd227ad657a0a802c2826477bf28aef984",
  final_content_root = "96c9e5994bdd94a57f7d4dc58b1b91783fa5f3b516dfeb579741e929a4e66b11",
  final_state_hash = "07e6254b3e33f730ae29a68214226fe00721fdf03b84fdd2098eecbad7731007",
  namespace_ref = "refs/namespaces/z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5/refs/cobs/xyz.onixresearch.cairn-change/5be37f1c60fcebd2db91101c38203b18eb32f877",
  bundle = {
    path = "octet-cob.bundle",
    blake3_hex = "e1e155b791cb7d75ee93d13d6716823812ab5ae3643cb429ca95a54df7102e4d",
    heads = [{
      target = "51bd1ec2dd218f068d1d55d01781e38c877dd063",
      ref_name = "refs/heads/main",
    }, {
      target = "649fa8cd227ad657a0a802c2826477bf28aef984",
      ref_name = "refs/namespaces/z6Mkg49NtQR2LyYRDCQFK4w1VVHqhypZSSRo7HsyuN7SV7v5/refs/cobs/xyz.onixresearch.cairn-change/5be37f1c60fcebd2db91101c38203b18eb32f877",
    }],
  },
  transport = {
    source_ref_target = "649fa8cd227ad657a0a802c2826477bf28aef984",
    destination_ref_target = "649fa8cd227ad657a0a802c2826477bf28aef984",
    source_remotes = [],
    destination_remotes = [],
    artifact_tree_blake3 = "blake3-QR+83p3e6U5jBNjE2R2cHA2Uepfdzpy3ugeEK9lFH3M=",
  },
  receipts = [{
    name = "bundle-reload.json",
    blake3_hex = "d65aef56a3a6a1581641858f30117c4b2d2caaa9c457ec5a93c993079a8cd0c0",
  }, {
    name = "checkout-execute.json",
    blake3_hex = "0928cb389884b06583b68e5b3e85d3b4e34992a4a85ba0992c0e92ddd06c0379",
  }, {
    name = "checkout-missing-changes-negative.json",
    blake3_hex = "fb2bd94ef86e00fc8af3716cb8089f75d1879bb95d036488e9e5fcb2e753e48b",
  }, {
    name = "checkout-preview.json",
    blake3_hex = "f0aa5078e4e827bb40446a1e90e212d53afb9b067a6b2f345c67e00e1dafc1e8",
  }, {
    name = "conflicts-secondary.json",
    blake3_hex = "b26b7f5352cccc746295e6d6d756b2d98989cddd15f7c7e6139cbe17e1f6444e",
  }, {
    name = "import-execute.json",
    blake3_hex = "feafb5fbbad14f21e5202d523faabad91f7810151d989b4798e172ec98945c89",
  }, {
    name = "import-preview.json",
    blake3_hex = "fd0d2d2dfc60e046389bfa86d53450fed2cee7af6c4bf9fb4afdf5da86e8f41c",
  }, {
    name = "status-secondary.json",
    blake3_hex = "323c2bf38ef244d32a3444a91c87de033119e493444938ab7766297f528bf9a1",
  }, {
    name = "transport.json",
    blake3_hex = "821902826fbe82e6d5bd0498af328d357f3358a46faa03ff36e55482478c6572",
  }, {
    name = "update-execute.json",
    blake3_hex = "00ad34dc3d4a6dcfce4326141985e59382411bdf5f7193e1c4f5e3a55bae572a",
  }, {
    name = "update-preview.json",
    blake3_hex = "fae645144cb1642e8e957537d71ac0c40ba5a5ee396e12f6433db8f4a9da43f8",
  }],
  required_non_claim_fragments = ["node participation", "synchronization", "convergence", "durability", "release eligibility"],
} | PilotManifest
