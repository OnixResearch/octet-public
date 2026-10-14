let contracts = import "contracts.ncl" in
{
  schema_version = "octet-verus-toolchain-profile/v1",
  profile_id = "octet-verus-0.2026.05.17.e479cce",
  verifier = {
    project = "verus-lang/verus",
    release = "0.2026.05.17.e479cce",
    source_revision = "e479cce36490b8fa4b0fd7755aa742aec354372c",
    release_url = "https://github.com/verus-lang/verus/releases/download/release/0.2026.05.17.e479cce/verus-0.2026.05.17.e479cce-x86-linux.zip",
    release_sha256_hex = "2a06101ab5fdcbc32073f1815e86a09c81d1f393c2d3501d35d0a24404d0a8e5",
    release_nix_sri = "sha256-KgYQGrX9y8Mgc/GBXoagnIHR85PC01AdNdCiRATQqOU=",
    rust_version = "1.95.0",
    rust_toolchain_label = "1.95.0-x86_64-unknown-linux-gnu",
    solver_id = "z3@verus-0.2026.05.17.e479cce",
  },
  formatter = {
    project = "verus-lang/verusfmt",
    version = "0.7.2",
    release_tag = "v0.7.2",
    source_revision = "610d4ee7fa4d5a8f0132aad70f8df07fd64adc87",
    self_update = "disabled-by-wrapper",
    assets = [
      {
        target = "x86_64-linux",
        file_name = "verusfmt-x86_64-unknown-linux-gnu.tar.xz",
        sha256_hex = "c5e0a8e07337055b2469d2878ddcc589da1f4be91348bb1917ce122ea46d4015",
        nix_sri = "sha256-xeCo4HM3BVskadKHjdzFidofS+kTSLsZF84SLqRtQBU=",
      },
      {
        target = "x86_64-darwin",
        file_name = "verusfmt-x86_64-apple-darwin.tar.xz",
        sha256_hex = "9208b3888a85b5138100aafe015391e0727e02b5dbc9e738d092515e77aa8808",
        nix_sri = "sha256-kgiziIqFtROBAKr+AVOR4HJ+ArXbyec40JJRXneqiAg=",
      },
      {
        target = "aarch64-darwin",
        file_name = "verusfmt-aarch64-apple-darwin.tar.xz",
        sha256_hex = "7bdf39bf01015c41deaa727f14dec41a8b755dbdecedde782d496cb191bfb8fe",
        nix_sri = "sha256-e985vwEBXEHeqnJ/FN7EGot1Xb3s7d54LUlssZG/uP4=",
      },
    ],
    unsupported_targets = ["aarch64-linux"],
  },
  compatibility = [
    {
      consumer = "octet",
      verifier_release = "0.2026.05.17.e479cce",
      status = "native",
      rationale = "Octet production proof rails use this exact reviewed verifier release.",
    },
    {
      consumer = "trellis",
      verifier_release = "0.2026.05.17.e479cce",
      status = "native",
      rationale = "Complete Trellis and all required-native corpus rows pass the May 17 profile; adapted rows remain separately identified and Trellis retains proof-source and receipt authority.",
    },
    {
      consumer = "onixos-lifecycle",
      verifier_release = "0.2026.04.03.21dfcd2",
      status = "local-pin",
      rationale = "OnixOS retains its published April package and receipt until the May 17 profile is rerun; OnixOS retains source-model and receipt authority.",
    },
    {
      consumer = "aspen-node-replication-pilot",
      verifier_release = "0.2026.04.03.21dfcd2",
      status = "local-pin",
      rationale = "Aspen retains its published April compatibility receipt until the May 17 profile is rerun; this does not authorize runtime adoption or transfer upstream proof claims.",
    },
    {
      consumer = "verus-belt",
      verifier_release = "0.2026.03.16.b1c2a95",
      status = "semantic-reference-only",
      rationale = "VerusBelt models a recorded snapshot and does not establish soundness for the Octet verifier release.",
    },
    {
      consumer = "aarch64-linux-formatter",
      verifier_release = "0.2026.05.17.e479cce",
      status = "unsupported",
      rationale = "The selected verusfmt release does not publish an aarch64-linux binary asset; source packaging remains future work.",
    },
  ],
  claim_scope = "toolchain-and-formatter-identity-inventory",
  non_claims = [
    "A compatibility row records reviewed identity relationships and does not transfer proof validity, verifier soundness, solver soundness, semantic coverage, or receipt authority across versions.",
    "Nix SHA-256 values satisfy fixed-output fetch requirements; Octet uses BLAKE3 for stack-owned binary, closure, artifact, and receipt identities after materialization.",
    "Formatter success proves only selected source formatting and does not prove Verus success, Rust correctness, whole-program safety, or release eligibility.",
  ],
} | contracts.VerusToolchainProfile
