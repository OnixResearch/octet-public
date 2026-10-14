# Aspen policy-artifact dogfood

This is a read-only downstream smoke for the generated Octet policy
artifact seam. It proves a consumer can receive the generated artifact bundle and
run `octet-standards check --policy-artifacts-dir ...` without reading the
Nickel registry directly.

## Scope

- Octet checkout: `/path/to/git/octet`
- Aspen checkout: `/path/to/git/aspen`
- Aspen HEAD during capture: `10bb6fb38` (`Document operator receipt evidence`)
- Aspen source roots checked:
  - `crates/aspen-core`
  - `crates/aspen-coordination`
  - `crates/aspen-rpc-handlers`
- Family: `embedded`
- Artifact files asserted non-empty:
  - `octet-policies.json`
  - `octet-check-config.toml`
  - `octet-seed-manifests.json`

The local Aspen worktree had one unrelated dirty file,
`tests/operator_receipts_docs.rs`, outside the checked source roots. The smoke
only reads source files and does not mutate Aspen.

## Reusable Nix check helper

Downstream flakes can package the same artifact-backed seam as a normal flake
check with `lib.mkPolicyArtifactConsumerCheck`:

```nix
checks.${system}.octet-standards-artifacts =
  octet.lib.mkPolicyArtifactConsumerCheck {
    inherit system;
    src = self;
    family = "embedded";
    sourceSubdirs = [
      "crates/aspen-core"
      "crates/aspen-coordination"
      "crates/aspen-rpc-handlers"
    ];
    expectedDiagnostics = 0;
    requiredPolicyIds = [ "octet.embedded.bounded-queues" ];
  };
```

The Octet repo keeps a fixture-backed regression for this helper at
`checks.x86_64-linux.policy-artifact-consumer-fixture`; the Aspen command below
is the real-consumer replay that motivated that reusable check surface.

## Replay command

```bash
set -euo pipefail
artifact_dir=$(mktemp -d)
out_json=$(mktemp)
nix run .#octet-policy-artifacts -- "$artifact_dir"
for f in octet-policies.json octet-check-config.toml octet-seed-manifests.json; do
  test -s "$artifact_dir/$f"
done
nix run .#octet-standards -- check \
  --policy-artifacts-dir "$artifact_dir" \
  --source /path/to/git/aspen/crates/aspen-core \
  --source /path/to/git/aspen/crates/aspen-coordination \
  --source /path/to/git/aspen/crates/aspen-rpc-handlers \
  --family embedded \
  --output json > "$out_json"
jq '{diagnostics:(.diagnostics|length), active_suppressions:(.active_suppressions|length), evaluated:(.evaluated_policy_ids|length), structural:any(.policy_evaluations[]; .target_enforcement_mode=="structural")}' "$out_json"
jq --exit-status '.diagnostics | length == 0' "$out_json" >/dev/null
jq --exit-status 'any(.policy_evaluations[]; .policy_id == "octet.embedded.bounded-queues" and .target_enforcement_mode == "structural")' "$out_json" >/dev/null
```

## Captured result

```text
artifact_dir=/tmp/tmp.RRfOhlihwA
this derivation will be built:
  /nix/store/r4p41wxyhxhx8hk85x63zakj4266094d-octet-policy-artifacts.drv
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/r4p41wxyhxhx8hk85x63zakj4266094d-octet-policy-artifacts.drv'...
{
  "diagnostics": 0,
  "active_suppressions": 0,
  "evaluated": 5,
  "structural": true
}
```

The remote-builder warning is environmental; Nix fell back to the local builder
and the command exited successfully.

## What this proves

- The generated artifact app emits the three expected consumer files.
- The standards checker can run from `--policy-artifacts-dir` without the Nickel
  registry path.
- Aspen's selected production crates have zero `embedded` family diagnostics for
  the artifact-backed bounded-queue structural policy.
- The JSON report still carries structural policy-evaluation metadata, including
  `octet.embedded.bounded-queues` with `target_enforcement_mode =
  "structural"`.
- The bounded-queue policy is now a hard structural gate: the generated bundle
  exports forbidden unbounded constructors, approved bounded APIs/wrapper scopes,
  capacity witnesses, and backpressure behavior witnesses; an Aspen scratch
  mutation that introduces a configured unbounded constructor must fail
  `checks.x86_64-linux.octet-policy-artifacts`.
