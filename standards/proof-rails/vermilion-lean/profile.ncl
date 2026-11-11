# r[impl octet.vermilion_proof_rail.provider_profile]
# Reviewed Vermilion-to-Lean provider profile.
#
# Pins observed from the Vermilion repository at
# 7c83bbca96940538eb282a67ca923893e5aa0e06 (README, Cargo.toml,
# lake-manifest.json, lean-toolchain, scripts/setup_verus.sh,
# scripts/run_example.sh). No provider process was executed by Octet.
let contracts = import "contracts.ncl" in
{
  schema_version = "octet-vermilion-lean-provider-profile/v1",
  profile_id = "octet-vermilion-lean-7c83bbc",
  provider_id = "vermilion-lean",
  claim_scope = "per-obligation-lean-kernel-theorem-over-pinned-verus-frontend",
  vermilion_revision = "7c83bbca96940538eb282a67ca923893e5aa0e06",
  vermilion_repository = "https://github.com/ilyasergey/vermilion",
  verus_fork = {
    repository = "https://github.com/ilyasergey/verus",
    branch = "dev",
    revision = "48600836424a79420589c56d86e720236a029811",
    vstd_revision = "48600836424a79420589c56d86e720236a029811",
  },
  translator = {
    name = "vermilion_direct+vrml_gen",
    revision = "7c83bbca96940538eb282a67ca923893e5aa0e06",
    argv = ["vermilion_direct", "<vir.bin>", "--out", "<ir.sexp>"],
  },
  lean = {
    toolchain = "leanprover/lean4:v4.33.0",
    mathlib_revision = "db584cd6d46c92f209a44c0f1c829460d327499d",
    lake_manifest_blake3 = "b3:a18fdfb10b7b53cc0368b088cd5d505ef811293ce639112a57db3f9b93a79479",
    packages = [
      { name = "smt", revision = "21aa44ce022fa37e0e40ff571ddde9b052a81c0c" },
      { name = "mathlib", revision = "db584cd6d46c92f209a44c0f1c829460d327499d" },
      { name = "cvc5", revision = "7e3365990661b697ccb30e92d6912f4cc6589322" },
      { name = "auto", revision = "a66de83d4eea7f2811613d7b24ce87cf46bb3ac2" },
      { name = "plausible", revision = "b7eb3304aeae834b12dda98993a37f6a41f6f0bb" },
      { name = "LeanSearchClient", revision = "5f4d51b81cbd3f6b32b156bfad9056621a040404" },
      { name = "importGraph", revision = "16f02aa7642864af59f1ff0e384a015994db9118" },
      { name = "proofwidgets", revision = "4be2e3d5087eeb272cf5a8853b8f9dd025ef5957" },
      { name = "aesop", revision = "3448c0bcc5ce01b2d1546e483ec3620e32df3d0e" },
      { name = "Qq", revision = "92c15be17b7caf78c2ad767ec40f89052d908d81" },
      { name = "batteries", revision = "4488d40d070b9700d4d5a6aa342f0d40c31b2a2d" },
      { name = "Cli", revision = "6130a47896ce867c6a4a55373441e59e565bad0f" },
    ],
  },
  checker = {
    checker_id = "vrml_check",
    revision = "7c83bbca96940538eb282a67ca923893e5aa0e06",
    argv = ["vrml_check", "<generated.json>", "--twin-check", "--skip-identical"],
    requires_twin_check = true,
  },
  environment_allowlist = [
    "PATH",
    "HOME",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "LEAN_PATH",
    "VERUS_VIR_EXPORT",
    "MATHLIB_NO_CACHE_ON_UPDATE",
  ],
  limits = {
    timeout_ms = 900000,
    stdout_max_bytes = 1048576,
    stderr_max_bytes = 1048576,
  },
  assumption_policy = {
    allowed_axioms = ["propext", "Classical.choice", "Quot.sound"],
    proof_holes = ["sorry", "admit"],
    requires_complete_declaration = true,
    require_twin_check = true,
  },
  non_claims = [
    "does not prove Vermilion, Verus, or Lean soundness",
    "does not prove the Verus front end, Vermilion lowering, or Lean generator correct",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not prove unsafe Rust or concurrent Rust",
    "does not establish release eligibility",
    "does not import matching Verus or Aeneas evidence as Vermilion evidence",
  ],
} | contracts.VermilionLeanProviderProfile
