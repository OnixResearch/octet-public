#!/usr/bin/env python3
"""Replay the local signed synthetic-root COB conflict/recovery fixture.

Requires an already-built Cairn CLI, public fixture-seed signer helper, b3sum,
Git, and the reviewed Nickel evaluator. No network or original Git objects.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

CHANGE = "exercise-cairn-cob-conflict-recovery"
WORKSPACE = "workspace:synthetic-cob-conflict"
BACKEND = "radicle-cob.v1"
NON_CLAIMS = ["semantic merge correctness", "synchronization", "distributed convergence",
              "durability", "release eligibility"]
CONTRACT = """let Receipt = { name | String, blake3_hex | String } in
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
"""


def nickel_value(value, indent=0):
    """Encode measured data as Nickel values, not a hand-edited JSON projection."""
    if isinstance(value, dict):
        pad = " " * indent
        return "{\n" + "".join(
            f"{pad}  {key} = {nickel_value(item, indent + 2)},\n"
            for key, item in value.items()) + pad + "}"
    if isinstance(value, list):
        return "[" + ", ".join(nickel_value(item, indent) for item in value) + "]"
    if isinstance(value, str):
        # Escape Nickel's interpolation opener as well as JSON string escapes.
        return json.dumps(value).replace("%{", "\\%{")
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    raise TypeError(f"unsupported measured Nickel value: {value!r}")


def replay(args):
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / "evidence").mkdir(exist_ok=True)
    (output / "heads").mkdir(exist_ok=True)
    cairn = str(Path(shutil.which(args.cairn) or args.cairn).resolve())
    keys = str(Path(shutil.which(args.keys) or args.keys).resolve())
    policy = Path(args.policy).resolve()
    env = os.environ.copy()
    for key in list(env):
        if key.startswith("GIT_") or key == "RAD_HOME":
            del env[key]
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
               GIT_TERMINAL_PROMPT="0")
    with tempfile.TemporaryDirectory(prefix="cairn-cob-conflict-replay-") as temporary:
        root = Path(temporary)
        env["HOME"] = str(root)

        def run(command, *, input=None, check=True):
            result = subprocess.run(command, cwd=root, env=env, input=input,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if check and result.returncode:
                raise RuntimeError(f"{command!r}: {result.stderr.decode()}")
            return result

        def git(store, *command, input=None):
            return run(["git", "--git-dir", store, *command], input=input).stdout.decode().strip()

        def init(store):
            run(["git", "init", "--bare", "--initial-branch=main", store])
            git(store, "config", "user.name", "Synthetic COB fixture")
            git(store, "config", "user.email", "cob-fixture@example.invalid")

        def receipt(name, command, *, verdict="PASS"):
            result = run([cairn, *command, "--policy", "policy.json"], check=False)
            destination = output / "evidence" / f"{name}.json"
            destination.write_bytes(result.stdout)
            try:
                value = json.loads(result.stdout)
            except ValueError as error:
                raise RuntimeError(f"{name}: CLI emitted no JSON: {result.stderr.decode()}") from error
            if value.get("verdict") != verdict or (verdict == "PASS" and result.returncode):
                raise RuntimeError(f"{name}: observed {value!r}; stderr={result.stderr.decode()}")
            return value

        def collaborate(name, action, workspace, store, *extra, verdict="PASS"):
            return receipt(name, ["collaboration", action, CHANGE, "--backend", BACKEND,
                                  "--root", workspace, "--git-repository", store, *extra],
                           verdict=verdict)

        shutil.copyfile(policy, root / "policy.json")
        actors = json.loads(run([keys, "keys"]).stdout)
        init("source.git")
        tree = git("source.git", "mktree", input=b"")
        base = git("source.git", "commit-tree", tree, input=b"Synthetic public COB fixture root\n")
        git("source.git", "update-ref", "refs/heads/main", base)
        assert git("source.git", "rev-list", "--count", "main") == "1"
        assert git("source.git", "ls-tree", "main") == ""
        change_dir = Path(".cairn/changes") / CHANGE
        original = root / "creator-workspace" / change_dir
        original.mkdir(parents=True)
        (original / "proposal.md").write_text("# Synthetic conflict drill\n\nCompare two authorized tasks updates from one signed basis.\n")
        (original / "design.md").write_text("# Design\n\nRetain both heads; inspect exact values; select one with both direct parents.\n")
        (original / "metadata.json").write_text("{}\n")
        (original / "tasks.md").write_text("# Tasks\n\n- [ ] Inspect both synthetic alternatives.\n")
        import_args = ["cob", "import", CHANGE, "--root", "creator-workspace",
                       "--git-repository", "source.git", "--keystore", "keys/creator",
                       "--workspace", WORKSPACE, "--delegate", actors["delegate"]]
        preview = receipt("import-preview", import_args)
        assert git("source.git", "for-each-ref", "--format=%(refname)") == "refs/heads/main"
        imported = receipt("import-execute", [*import_args, "--execute"])
        assert imported["plan_hash"] == preview["plan_hash"]
        assert imported["actor"] == actors["creator"]
        assert imported["delegates"] == [actors["delegate"]]
        object_id = imported["object_id"]

        def actor_ref(actor):
            return f"refs/namespaces/{actor}/refs/cobs/xyz.onixresearch.cairn-change/{object_id}"

        creator_ref = actor_ref(actors["creator"])
        delegate_ref = actor_ref(actors["delegate"])
        # Fork before either update: both stores contain precisely the signed import basis.
        init("delegate.git")
        git("delegate.git", "fetch", str(root / "source.git"), "refs/heads/main:refs/heads/main",
            f"{creator_ref}:{creator_ref}")
        shutil.copytree(root / "creator-workspace", root / "delegate-workspace")
        heads = {}
        for label, store in (("creator", "source.git"), ("delegate", "delegate.git")):
            workspace = f"{label}-workspace"
            content = f"# Tasks\n\n- [ ] Inspect the {label}'s synthetic alternative.\n"
            tasks = root / workspace / change_dir / "tasks.md"
            tasks.write_text(content)
            shutil.copyfile(tasks, output / "heads" / f"{label}-tasks.md")
            extra = ["--keystore", f"keys/{label}"]
            initial_refs = git(store, "show-ref")
            plan = collaborate(f"{label}-update-preview", "update", workspace, store, *extra)
            assert git(store, "show-ref") == initial_refs
            update = collaborate(f"{label}-update-execute", "update", workspace, store,
                                 *extra, "--execute")
            assert plan["plan_hash"] == update["plan_hash"]
            assert update["input_state_hash"] == imported["state_hash"]
            assert update["updates"][0]["basis"] == [imported["operation_id"]]
            assert update["actor"]["authority_class"] == ("signed_creator" if label == "creator" else "embedded_delegate")
            heads[label] = dict(actor=actors[label], ref_name=actor_ref(actors[label]),
                                operation_id=update["result_operation_id"],
                                content_blake3=hash_file(tasks), content_bytes=tasks.stat().st_size,
                                content_root=update["result_content_root"],
                                state_hash=update["result_state_hash"],
                                retained_path=f"heads/{label}-tasks.md")
        init("inspection.git")
        git("inspection.git", "fetch", str(root / "source.git"), "refs/heads/main:refs/heads/main",
            f"{creator_ref}:{creator_ref}")
        git("inspection.git", "fetch", str(root / "delegate.git"), f"{delegate_ref}:{delegate_ref}")
        shutil.copytree(root / "creator-workspace", root / "inspection-workspace")
        conflict = collaborate("conflicts-metadata", "conflicts", "inspection-workspace", "inspection.git")
        assert conflict["conflicted"] and conflict["shown_head"] is None
        assert len(conflict["artifact_conflicts"][0]["heads"]) == 2
        for label, head in heads.items():
            shown = collaborate(f"conflicts-show-{label}", "conflicts", "inspection-workspace",
                                "inspection.git", "--show-head", f"tasks={head['operation_id']}")
            assert shown["shown_head"]["content"] == (output / head["retained_path"]).read_text()
            assert shown["shown_head"]["content_blake3"] == head["content_blake3"]
        conflicting_refs = git("inspection.git", "show-ref")
        refused = collaborate("update-conflict-negative", "update", "inspection-workspace",
                              "inspection.git", "--keystore", "keys/creator", verdict="FAIL")
        assert refused["blocked"] and refused["dry_run"] and not refused["mutated"]
        assert git("inspection.git", "show-ref") == conflicting_refs
        selection = ["--keystore", "keys/creator", "--choose-head", f"tasks={heads['creator']['operation_id']}"]
        plan = collaborate("resolve-preview", "resolve", "inspection-workspace", "inspection.git", *selection)
        assert git("inspection.git", "show-ref") == conflicting_refs
        resolved = collaborate("resolve-execute", "resolve", "inspection-workspace", "inspection.git", *selection, "--execute")
        assert plan["plan_hash"] == resolved["plan_hash"]
        resolution_oid = resolved["result_operation_id"]
        parents = git("inspection.git", "show", "--format=%P", "--no-patch", resolution_oid).split()
        competing = [heads["creator"]["operation_id"], heads["delegate"]["operation_id"]]
        assert set(parents) == set(competing) and len(parents) == 2
        assert set(resolved["actions"][0]["basis"]) == set(competing)
        git("inspection.git", "merge-base", "--is-ancestor", heads["delegate"]["operation_id"], resolution_oid)
        git_commits = int(git("inspection.git", "rev-list", "--all", "--count"))
        git_roots = git("inspection.git", "rev-list", "--all", "--max-parents=0").splitlines()
        assert git_commits == 5 and git_roots == [base]
        assert git("inspection.git", "ls-tree", base) == ""
        bundle = "octet-cob-conflict.bundle"
        git("inspection.git", "bundle", "create", str(root / bundle), "refs/heads/main", creator_ref, delegate_ref)
        git("inspection.git", "bundle", "verify", str(root / bundle))
        shutil.copyfile(root / bundle, output / bundle)
        bundle_heads = [dict(target=line.split()[0], ref_name=line.split()[1])
                        for line in git("inspection.git", "bundle", "list-heads", str(root / bundle)).splitlines()]
        assert len(bundle_heads) == 3
        init("reload.git")
        git("reload.git", "fetch", str(root / bundle), "refs/heads/main:refs/heads/main",
            f"{creator_ref}:{creator_ref}", f"{delegate_ref}:{delegate_ref}")
        (root / "reload-workspace/.cairn/changes").mkdir(parents=True)
        before = collaborate("status-third-before-checkout", "status", "reload-workspace", "reload.git")
        assert before["local_destination"] == "absent"
        reloaded = collaborate("conflicts-resolved-third", "conflicts", "reload-workspace", "reload.git")
        assert not reloaded["conflicted"] and reloaded["state_hash"] == resolved["result_state_hash"]
        checkout_plan = collaborate("checkout-third-preview", "checkout", "reload-workspace", "reload.git")
        assert not (root / "reload-workspace" / change_dir).exists()
        checkout = collaborate("checkout-third-execute", "checkout", "reload-workspace", "reload.git", "--execute")
        assert checkout["plan_hash"] == checkout_plan["plan_hash"]
        assert checkout["post_checkout_destination"] == "current"
        assert (root / "reload-workspace" / change_dir / "tasks.md").read_bytes() == (output / heads["creator"]["retained_path"]).read_bytes()
        after = collaborate("status-third-after-checkout", "status", "reload-workspace", "reload.git")
        assert after["state_hash"] == resolved["result_state_hash"]
        assert after["lifecycle"]["condition"] == "stale"
        remotes = {store: git(store, "remote").splitlines()
                   for store in ("source.git", "delegate.git", "inspection.git", "reload.git")}
        assert all(not values for values in remotes.values())
        # Transport is measured Git evidence, not an invented CLI receipt.
        transport = dict(cairn_commit=args.cairn_commit, synthetic_base_git_oid=base,
                         object_id=object_id,
                         creator=dict(actor=actors["creator"], ref_name=creator_ref,
                                      resolved_target=git("inspection.git", "rev-parse", creator_ref)),
                         delegate=dict(actor=actors["delegate"], ref_name=delegate_ref,
                                       resolved_target=git("inspection.git", "rev-parse", delegate_ref)),
                         resolution=dict(direct_parents=parents, losing_revision_reachable=True),
                         source_remotes=remotes["source.git"], convergence_remotes=remotes["inspection.git"],
                         reload_remotes=remotes["reload.git"], valid=True,
                         non_claims=[
                             "shell-measured Git observations, not a Cairn-emitted receipt or Radicle runtime observation",
                             "local Git bundle transport and reload observations do not prove semantic merge correctness, synchronization, distributed convergence, durability, or release eligibility",
                         ])
        transport_contract = """let Actor = { actor | String, ref_name | String, resolved_target | String } in
let Transport = {
  cairn_commit | String, synthetic_base_git_oid | String, object_id | String,
  creator | Actor, delegate | Actor,
  resolution | { direct_parents | Array String, losing_revision_reachable | Bool },
  source_remotes | Array String, convergence_remotes | Array String,
  reload_remotes | Array String, valid | Bool, non_claims | Array String,
} in
"""
        transport_path = output / "transport.ncl"
        transport_path.write_text(transport_contract + nickel_value(transport) + " | Transport\n")
        transport_json = run([args.nickel, "export", "--format", "json", str(transport_path)]).stdout
        assert json.loads(transport_json) == transport
        (output / "evidence/transport.json").write_bytes(transport_json)
        receipts = sorted((output / "evidence").glob("*.json"))
        assert len(receipts) == 18
        emitted_non_claims = "\n".join(json.dumps(json.loads(path.read_bytes()).get("non_claims", []))
                                      for path in receipts)
        assert all(fragment in emitted_non_claims for fragment in NON_CLAIMS)
        manifest = dict(schema="octet.cairn-cob-conflict-pilot.v2", change=CHANGE,
                        workspace_id=WORKSPACE, backend=BACKEND, cairn_commit=args.cairn_commit,
                        synthetic_base_git_oid=base, policy_hash=imported["policy_hash"],
                        object_id=object_id, root_operation_id=imported["operation_id"],
                        root_content_root=imported["content_root"], root_state_hash=imported["state_hash"],
                        creator=heads["creator"], delegate=heads["delegate"],
                        conflict_state_hash=conflict["state_hash"],
                        resolution=dict(operation_id=resolution_oid, selected_revision=competing[0],
                                        losing_revision=competing[1], direct_parents=parents,
                                        content_root=resolved["result_content_root"], state_hash=resolved["result_state_hash"]),
                        resolved_lifecycle_condition=after["lifecycle"]["condition"],
                        bundle=dict(path=bundle, blake3_hex=hash_file(output / bundle), heads=bundle_heads),
                        receipts=[dict(name=path.name, blake3_hex=hash_file(path)) for path in receipts],
                        required_non_claim_fragments=NON_CLAIMS)
        manifest_path = output / "manifest.ncl"
        manifest_path.write_text(CONTRACT + nickel_value(manifest) + " | ConflictPilotManifest\n")
        exported = run([args.nickel, "export", "--format", "json", str(manifest_path)]).stdout
        assert json.loads(exported) == manifest
        (output / "manifest.json").write_bytes(exported)
        print(json.dumps(dict(output=str(output), synthetic_base_git_oid=base,
                              object_id=object_id, competing_heads=competing,
                              resolution=resolution_oid, lifecycle=after["lifecycle"]["condition"],
                              receipts=len(receipts), remotes=remotes,
                              git_commits=git_commits, git_roots=git_roots), indent=2))


def hash_file(path):
    return subprocess.check_output(["b3sum", str(path)], text=True).split()[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cairn", required=True)
    parser.add_argument("--keys", required=True)
    parser.add_argument("--policy", required=True)
    parser.add_argument("--cairn-commit", required=True)
    parser.add_argument("--nickel", default="nickel")
    parser.add_argument("--output", default=str(Path(__file__).resolve().parents[1] / "fixtures/cairn-cob-conflict-pilot"))
    args = parser.parse_args()
    args.nickel = str(Path(shutil.which(args.nickel) or args.nickel).resolve())
    if len(args.cairn_commit) != 40 or any(char not in "0123456789abcdef" for char in args.cairn_commit):
        parser.error("--cairn-commit must be an immutable full SHA-1 source identity")
    version = subprocess.check_output([args.nickel, "--version"], text=True).strip()
    if version != "nickel-lang-cli nickel 1.17.0 (rev 1320a98)":
        parser.error(f"unreviewed Nickel evaluator: {version}")
    replay(args)


if __name__ == "__main__":
    main()
