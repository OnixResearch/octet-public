# Octet owns this bounded producer job. It selects existing contracts, not new
# theorem policy; Mantle may realize its complete exported ordinary drv graph.
let consumer = import "consumer-profile.ncl" in
let providerProfile = import "profile.ncl" in
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Job = {
  schema_version | String | std.contract.from_predicate (fun value => value == "octet-mantle-producer-job/v1"),
  owner | String,
  job_id | NonEmpty,
  system | NonEmpty,
  package_attribute | NonEmpty,
  profile | { path | NonEmpty, identity | NonEmpty },
  provider | { path | NonEmpty, identity | NonEmpty },
  source | { root | NonEmpty, paths | Array NonEmpty, network_allowed | Bool },
  execution | { command_argv | Array String, artifact_verify_argv | Array String, rail_id | NonEmpty, network_allowed | Bool },
  tools | { closure_format | NonEmpty, graph_formats | Array NonEmpty, dependencies | Array NonEmpty, solver | NonEmpty },
  limits | { cores | Number, timeout_ms | Number, stdout_max_bytes | Number, stderr_max_bytes | Number },
  outputs | Array NonEmpty,
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-mantle-producer-job/v1",
  owner = "Octet",
  job_id = "octet.aeneas.external-consumer.mantle.v1",
  system = "x86_64-linux",
  package_attribute = "packages.x86_64-linux.octet-mantle-aeneas-consumer-job",
  profile = {
    path = "standards/proof-rails/aeneas-lean/generated/consumer-profile.json",
    identity = "b3:86351a1b7a93d5fbcacc20aa97ad8fff4c9d15b670f47f55988e5ac2deb66291",
  },
  provider = {
    path = "standards/proof-rails/aeneas-lean/generated/profile.json",
    identity = consumer.provider.profile_identity,
  },
  source = {
    root = "standards/proof-rails/aeneas-lean/fixtures/external-consumer",
    paths = ["Cargo.toml", "src/lib.rs", "Theorem.lean"],
    network_allowed = false,
  },
  execution = {
    command_argv = ["cargo-octet", "prove", "--policy-artifacts-dir", "$POLICY", "--source", "src/lib.rs", "--rail", "octet.aeneas_consumer_contract.external-fixture#rail-0", "--artifact-dir", "$ARTIFACTS", "--output-format", "json"],
    artifact_verify_argv = ["cargo-octet", "artifact", "verify", "--artifact-dir", "$ARTIFACTS", "--format", "json", "--emit-valence-provenance", "$OUTPUT/valence-provenance.json"],
    rail_id = "octet.aeneas_consumer_contract.external-fixture#rail-0",
    network_allowed = false,
  },
  tools = {
    closure_format = "nix-derivation-show-recursive-json; preserve raw bytes and flake.lock; legacy uses absolute inputDrvs/inputSrcs, v4 uses basename derivations and inputs.drvs/inputs.srcs with explicit version 4",
    graph_formats = ["nix-derivation-show-legacy", "nix-derivation-show-v4"],
    dependencies = ["cargo-octet", "octet-policy-artifacts", "octet-aeneas-lean-provider-closure", providerProfile.aeneas_revision, providerProfile.charon_revision, providerProfile.lean_toolchain, "flake.lock:mathlib-src", "flake.lock:nixpkgs", "flake.lock:nixpkgs:stdenv.cc", "flake.lock:aeneas"],
    solver = "none: this selected Aeneas/Charon/Lean rail invokes no external SMT solver; imported Verus evidence is not accepted by this job",
  },
  limits = {
    # OCaml 5.2 reads CPU affinity, not NIX_BUILD_CORES, for domain fanout.
    cores = 32,
    timeout_ms = consumer.resource_policy.timeout_ms,
    stdout_max_bytes = consumer.resource_policy.stdout_max_bytes,
    stderr_max_bytes = consumer.resource_policy.stderr_max_bytes,
  },
  outputs = ["artifacts", "producer-artifact-verification.json", "producer-prove.json", "consumer-profile.json", "provider-identity.json", "job.json", "source", "external-profile.json", "external-receipt.json", "source-revision", "flake.lock", "valence-provenance.json"],
  non_claims = consumer.non_claims @ [
    "Nix evaluation supplies a frozen ordinary derivation graph; it is not theorem admission",
    "Mantle realizes and signs build outputs, not proof truth or external acceptance policy",
    "A cached producer output preserves its recorded external acceptance; cache reuse is not a fresh verifier execution",
    "The recursive tool graph includes declared bootstrap/binary acquisition inputs and does not claim seed-free bootstrap",
  ],
} | Job
