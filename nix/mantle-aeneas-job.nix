# Octet-authored ordinary derivation. No verifier policy lives in Mantle.
{ pkgs, nickel, provider, cargoOctet, policyArtifacts, rustToolchain, sourceRoot, sourceRevision }:
let
  consumerProfile = sourceRoot + "/standards/proof-rails/aeneas-lean/generated/consumer-profile.json";
  providerRoot = "${provider}/share/octet/aeneas-lean";
  jobPolicy = sourceRoot + "/standards/proof-rails/aeneas-lean/mantle-job.ncl";
in
pkgs.runCommand "octet-mantle-aeneas-consumer-job-v1" {
  nativeBuildInputs = [ nickel pkgs.coreutils pkgs.jq pkgs.bash pkgs.stdenv.cc ];
  # Avoid impure/local preference: this derivation must be sandboxable by Mantle.
  preferLocalBuild = false;
  allowSubstitutes = true;
} ''
  set -euo pipefail
  stage=prepare-inputs
  # Nix removes failed output paths. Mirror original diagnostics into the build
  # log and keep a copy in TMPDIR for --keep-failed, without admitting an output.
  retain_failure() {
    status="$1"
    if [ "$status" -eq 0 ]; then
      return
    fi
    set +e
    printf 'Octet Mantle producer failed: stage=%s exit=%s\n' "$stage" "$status" >&2
    diagnostics="$TMPDIR/producer-diagnostics"
    mkdir -p "$diagnostics"
    printf 'stage=%s\nexit=%s\n' "$stage" "$status" > "$diagnostics/failure.txt"
    for report in "$out/producer-prove.json" "$out/producer-artifact-verification.json"; do
      if [ -f "$report" ]; then
        cp "$report" "$diagnostics/"
        printf '\n--- %s ---\n' "$report" >&2
        cat "$report" >&2
      fi
    done
    if [ -d "$out/artifacts" ]; then
      cp -R "$out/artifacts" "$diagnostics/artifacts"
      for log in "$out"/artifacts/proofs/*.stdout "$out"/artifacts/proofs/*.stderr "$out"/artifacts/proofs/*.receipt; do
        if [ -f "$log" ]; then
          printf '\n--- %s ---\n' "$log" >&2
          cat "$log" >&2
        fi
      done
    fi
    printf '\nOriginal producer diagnostics retained at %s (requires Nix --keep-failed).\n' "$diagnostics" >&2
    exit "$status"
  }
  trap 'retain_failure "$?"' EXIT
  export HOME="$TMPDIR/home"
  mkdir -p "$HOME" "$out/source" "$out/artifacts"
  cp -R ${sourceRoot}/standards/proof-rails/aeneas-lean/fixtures/external-consumer/. "$out/source/"
  chmod -R u+w "$out/source"
  # Expected artifacts are not producer inputs. Only actual execution may create them.
  rm -rf "$out/source/expected"
  cp -R "$out/source" "$TMPDIR/source"
  cp ${consumerProfile} "$out/consumer-profile.json"
  cp ${providerRoot}/identity.json "$out/provider-identity.json"
  cp ${sourceRoot}/flake.lock "$out/flake.lock"
  nickel export --format json ${jobPolicy} > "$out/job.json"
  # Fail closed if committed runtime bytes drift from their Nickel source.
  stage=consumer-profile-freshness
  nickel export --format json ${sourceRoot}/standards/proof-rails/aeneas-lean/consumer-profile.ncl > "$TMPDIR/consumer-profile.json"
  cmp "$TMPDIR/consumer-profile.json" "$out/consumer-profile.json"
  printf '%s\n' '${sourceRevision}' > "$out/source-revision"
  stage=export-policy
  ${policyArtifacts}/bin/octet-policy-artifacts "$TMPDIR/policy"
  stage=bound-worker-affinity
  # OCaml 5.2 recommends domains from CPU affinity, ignoring NIX_BUILD_CORES.
  # Keep the inherited CPU set and restrict it to this job's reviewed budget.
  core_budget=$(jq --exit-status '.limits.cores' "$out/job.json")
  nix_cores="''${NIX_BUILD_CORES:-0}"
  # Nix defines zero as all available cores; the job's own cap still applies.
  if [ "$nix_cores" -gt 0 ] && [ "$nix_cores" -lt "$core_budget" ]; then
    core_budget="$nix_cores"
  fi
  affinity=$(LC_ALL=C ${pkgs.util-linux}/bin/taskset --pid --cpu-list "$$")
  affinity="''${affinity##*: }"
  IFS=, read -r -a cpu_ranges <<< "$affinity"
  cpu_list=
  cpu_count=0
  for range in "''${cpu_ranges[@]}"; do
    first="''${range%-*}"
    last="''${range#*-}"
    for ((cpu=first; cpu<=last && cpu_count<core_budget; cpu++)); do
      cpu_list="''${cpu_list:+$cpu_list,}$cpu"
      cpu_count=$((cpu_count + 1))
    done
    if [ "$cpu_count" -ge "$core_budget" ]; then
      break
    fi
  done
  test "$cpu_count" -gt 0
  # Children, including the unchanged bwrap/provider argv, inherit this mask.
  ${pkgs.util-linux}/bin/taskset --pid --cpu-list "$cpu_list" "$$" >&2
  export PATH="${pkgs.lib.makeBinPath [ rustToolchain pkgs.stdenv.cc pkgs.coreutils pkgs.bash pkgs.jq ]}"
  export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ rustToolchain ]}"
  export CHARON_TOOLCHAIN_IS_IN_PATH=1
  export CARGO_NET_OFFLINE=true
  export MATHLIB_NO_CACHE_ON_UPDATE=1
  export OCTET_AENEAS_CHARON="${providerRoot}/charon"
  export OCTET_AENEAS_AENEAS="${providerRoot}/aeneas"
  export OCTET_AENEAS_LEAN="${providerRoot}/lake"
  export OCTET_AENEAS_LEAN_LIBRARY="${providerRoot}/aeneas-lean-library"
  export OCTET_AENEAS_NETWORK_SANDBOX="${providerRoot}/bwrap"
  cd "$TMPDIR/source"
  stage=prove
  ${cargoOctet}/bin/cargo-octet prove \
    --policy-artifacts-dir "$TMPDIR/policy" \
    --source src/lib.rs \
    --rail octet.aeneas_consumer_contract.external-fixture#rail-0 \
    --artifact-dir "$out/artifacts" \
    --output-format json > "$out/producer-prove.json"
  stage=artifact-verification
  ${cargoOctet}/bin/cargo-octet artifact verify \
    --artifact-dir "$out/artifacts" --format json \
    --emit-valence-provenance "$out/valence-provenance.json" \
    > "$out/producer-artifact-verification.json"
  stage=artifact-report-contract
  jq --exit-status '.schema_version == 1 and .status == "valid" and (.diagnostics | length == 0)' \
    "$out/producer-artifact-verification.json" >/dev/null
  receipt="$out/artifacts/proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.receipt"
  stage=consumer-receipt-contract
  jq --exit-status '.schema_version == "octet-formal-proof-envelope/v1" and .status == "passed" and .consumer.outcome == "supported" and .consumer.verification_role == "property"' "$receipt" >/dev/null
  stage=preserve-original-output
  # Copy original bytes, never reconstruct a receipt from process exit status.
  cp "$receipt" "$out/external-receipt.json"
  cp ${consumerProfile} "$out/external-profile.json"
  # Preserve every original report/envelope locator for full consumer replay.
  cp -R "$out/artifacts/." "$out/"
  mkdir -p "$out/src"
  cp "$out/source/src/lib.rs" "$out/src/lib.rs"
''
