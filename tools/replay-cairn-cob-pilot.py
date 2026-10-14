#!/usr/bin/env python3
"""Replay the bounded, local-only synthetic Cairn COB pilot with explicit tools."""

import argparse
import base64
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


CHANGE = "adopt-cairn-cob-collaboration"
WORKSPACE = "workspace:octet"
BACKEND = "radicle-cob.v1"
RECEIPTS = (
    "bundle-reload.json", "checkout-execute.json",
    "checkout-missing-changes-negative.json", "checkout-preview.json",
    "conflicts-secondary.json", "import-execute.json", "import-preview.json",
    "status-secondary.json", "transport.json", "update-execute.json",
    "update-preview.json",
)
NON_CLAIMS = ["node participation", "synchronization", "convergence", "durability", "release eligibility"]
TRANSPORT_CONTRACT = """let TransportObservation = {
  schema | String,
  cairn_commit | String,
  actor | String,
  object_id | String,
  base_git_oid | String,
  namespace_ref | String,
  source_ref_target | String,
  destination_ref_target | String,
  source_branch_target | String,
  destination_branch_target | String,
  source_remotes | Array String,
  destination_remotes | Array String,
  destination_tags | Array String,
  artifact_tree_blake3 | String,
  artifact_diff_empty | Bool,
  mutated_by_cairn | Bool,
  valid | Bool,
  non_claims | Array String,
} in
"""


def nickel(value, indent=0):
    """Encode measured data as Nickel values, never as a JSON config source."""
    if isinstance(value, dict):
        rows = [" " * (indent + 2) + key + " = " + nickel(item, indent + 2) + ","
                for key, item in value.items()]
        return "{\n" + "\n".join(rows) + "\n" + " " * indent + "}"
    if isinstance(value, list):
        return "[" + ", ".join(nickel(item, indent) for item in value) + "]"
    if isinstance(value, str):
        return json.dumps(value).replace("%{", "\\%{")
    if isinstance(value, bool):
        return "true" if value else "false"
    raise TypeError(type(value))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("cairn", "keys", "policy", "nickel", "cairn-commit"):
        parser.add_argument("--" + option, required=True)
    parser.add_argument("--output", type=Path,
                        default=Path(__file__).resolve().parents[1] / "fixtures/cairn-cob-pilot")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.cairn_commit):
        parser.error("--cairn-commit must be the full reviewed source commit")
    tools = {name: str(Path(getattr(args, name)).resolve(strict=True))
             for name in ("cairn", "keys", "policy", "nickel")}
    fixture = args.output.resolve()
    fixture.mkdir(parents=True, exist_ok=True)
    # Preserve the reviewed, human-authored manifest contract, not its old identities.
    template = Path(__file__).resolve().parents[1] / "fixtures/cairn-cob-pilot/manifest.ncl"
    contract = template.read_text().split("\n{\n", 1)[0] + "\n"
    with tempfile.TemporaryDirectory(prefix="octet-cob-pilot-") as temporary:
        root = Path(temporary)
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   HOME=str(root / "home"), LC_ALL="C", TZ="UTC")
        (root / "home").mkdir()

        def run(command, data=None, extra_env=None):
            result = subprocess.run([str(item) for item in command], cwd=root, env=env | (extra_env or {}),
                                    input=data, capture_output=True, check=False)
            if result.returncode:
                raise RuntimeError(f"{command}: exit {result.returncode}\n{result.stderr.decode()}")
            return result.stdout

        def git(repository, *arguments):
            return run(["git", "-C", repository, *arguments]).decode().strip()

        def digest(data):
            return run(["b3sum", "--no-names"], data).decode().strip()

        version = run([tools["nickel"], "--version"]).decode().strip()
        if version != "nickel-lang-cli nickel 1.17.0 (rev 1320a98)":
            raise RuntimeError(f"unreviewed Nickel evaluator: {version}")
        shutil.copyfile(tools["policy"], root / "policy.json")
        actors = json.loads(run([tools["keys"], "keys"]))
        for repository in ("primary", "secondary", "reload"):
            run(["git", "init", "--initial-branch=main", "--template=", repository])
            git(repository, "config", "user.name", "Synthetic COB fixture")
            git(repository, "config", "user.email", "cob-fixture@example.invalid")
            git(repository, "config", "commit.gpgsign", "false")
        run(["git", "-C", "primary", "commit", "--allow-empty", "-m", "Synthetic public COB fixture root"],
            extra_env={"GIT_AUTHOR_DATE": "2000-01-01T00:00:00Z", "GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"})
        base = git("primary", "rev-parse", "HEAD")
        assert git("primary", "rev-list", "--count", "HEAD") == "1"
        assert not git("primary", "ls-tree", "-r", "HEAD")
        active = Path(".cairn/changes") / CHANGE
        source = root / "primary" / active
        source.mkdir(parents=True)
        (source / "proposal.md").write_text("# Synthetic local collaboration\n\nExercise signed local import and checkout only.\n")
        (source / "tasks.md").write_text("# Tasks\n\n- [ ] Observe the synthetic local update.\n")
        evidence = root / "output/evidence"
        evidence.mkdir(parents=True)

        def observe(name, command, repository, execute=False):
            arguments = [tools["cairn"], *command, CHANGE, "--root", repository,
                         "--git-repository", repository, "--policy", "policy.json"]
            if command == ["cob", "import"]:
                arguments += ["--workspace", WORKSPACE, "--keystore", "keys/creator"]
            elif command == ["collaboration", "update"]:
                arguments += ["--backend", BACKEND, "--keystore", "keys/creator"]
            else:
                arguments += ["--backend", BACKEND]
            if execute:
                arguments += ["--execute"]
            raw = run(arguments)
            value = json.loads(raw)
            (evidence / name).write_bytes(raw)
            negative = name == "checkout-missing-changes-negative.json"
            if value["verdict"] != ("FAIL" if negative else "PASS"):
                refusal = Path(tempfile.mkdtemp(prefix="octet-cob-pilot-refusal-"))
                shutil.copytree(root / "output", refusal, dirs_exist_ok=True)
                raise RuntimeError(f"unexpected {name} verdict; exact receipt bytes retained at {refusal}: {value}")
            print(f"{name}: verdict={value['verdict']} mutated={value.get('mutated')}")
            return value

        imported_preview = observe("import-preview.json", ["cob", "import"], "primary")
        assert not imported_preview["mutated"] and imported_preview["object_id"] is None
        imported = observe("import-execute.json", ["cob", "import"], "primary", True)
        assert imported["mutated"] and imported["actor"] == actors["creator"]
        assert imported["repository_head_oid"] == base
        for field in ("plan_hash", "input_hash", "content_root"):
            assert imported[field] == imported_preview[field]
        (source / "tasks.md").write_text("# Tasks\n\n- [x] Observe the synthetic local update.\n")
        updated_preview = observe("update-preview.json", ["collaboration", "update"], "primary")
        assert not updated_preview["mutated"] and not updated_preview["noop"]
        updated = observe("update-execute.json", ["collaboration", "update"], "primary", True)
        assert updated["mutated"] and updated["changed_artifacts"] == ["tasks"]
        assert updated["actor"]["id"] == imported["actor"]
        assert updated["object_id"] == imported["object_id"]
        assert updated["plan_hash"] == updated_preview["plan_hash"]
        assert updated["result_content_root"] == updated_preview["expected_content_root"]
        namespace = f"refs/namespaces/{imported['actor']}/refs/cobs/xyz.onixresearch.cairn-change/{imported['object_id']}"
        tip = git("primary", "rev-parse", namespace)
        assert tip == updated["result_operation_id"]
        bundle = root / "output/octet-cob.bundle"
        git("primary", "bundle", "create", str(bundle), "refs/heads/main", namespace)
        heads = [{"target": oid, "ref_name": ref}
                 for oid, ref in (line.split() for line in git("primary", "bundle", "list-heads", str(bundle)).splitlines())]
        assert {head["ref_name"] for head in heads} == {"refs/heads/main", namespace}
        for destination in ("secondary", "reload"):
            git(destination, "bundle", "verify", str(bundle))
            git(destination, "fetch", "--no-tags", "--update-head-ok", str(bundle),
                "refs/heads/main:refs/heads/main", f"{namespace}:{namespace}")
            git(destination, "reset", "--hard", "main")
            assert not (root / destination / ".git/objects/info/alternates").exists()
        status = observe("status-secondary.json", ["collaboration", "status"], "secondary")
        negative = observe("checkout-missing-changes-negative.json", ["collaboration", "checkout"], "secondary", True)
        assert negative["blocked"] and not negative["mutated"] and negative["blockers"]
        assert not (root / "secondary" / active).exists()
        (root / "secondary/.cairn/changes").mkdir(parents=True)
        checkout_preview = observe("checkout-preview.json", ["collaboration", "checkout"], "secondary")
        assert not checkout_preview["mutated"] and not (root / "secondary" / active).exists()
        checkout = observe("checkout-execute.json", ["collaboration", "checkout"], "secondary", True)
        assert checkout["mutated"] and checkout["post_checkout_destination"] == "current"
        conflicts = observe("conflicts-secondary.json", ["collaboration", "conflicts"], "secondary")
        assert not conflicts["conflicted"]
        reload_status = observe("bundle-reload.json", ["collaboration", "status"], "reload")
        for observation in (status, checkout_preview, checkout, conflicts, reload_status):
            assert observation["state_hash"] == updated["result_state_hash"]
            assert observation["object_id"] == imported["object_id"]
        assert reload_status["authority"]["creator"] == actors["creator"]

        def artifacts(directory):
            return {path.relative_to(directory).as_posix(): path.read_bytes().decode("utf-8")
                    for path in sorted(directory.rglob("*")) if path.is_file()}

        source_files = artifacts(source)
        destination_files = artifacts(root / "secondary" / active)
        assert source_files == destination_files
        tree_hash = digest(json.dumps(source_files, sort_keys=True, separators=(",", ":")).encode())
        transport = {
            "schema": "octet.cairn-cob-pilot-transport.v1",
            "cairn_commit": args.cairn_commit, "actor": imported["actor"],
            "object_id": imported["object_id"], "base_git_oid": base, "namespace_ref": namespace,
            "source_ref_target": git("primary", "rev-parse", namespace),
            "destination_ref_target": git("secondary", "rev-parse", namespace),
            "source_branch_target": git("primary", "rev-parse", "main"),
            "destination_branch_target": git("secondary", "rev-parse", "main"),
            "source_remotes": git("primary", "remote").splitlines(),
            "destination_remotes": git("secondary", "remote").splitlines(),
            "destination_tags": git("secondary", "tag", "--list").splitlines(),
            "artifact_tree_blake3": "blake3-" + base64.b64encode(bytes.fromhex(tree_hash)).decode(),
            "artifact_diff_empty": source_files == destination_files,
            "mutated_by_cairn": False,
            "valid": True,
            "non_claims": ["Local bundle transfer is Git transport, not Cairn synchronization, node participation, convergence, durability, or release eligibility."],
        }
        assert not transport["source_remotes"] and not transport["destination_remotes"] and not transport["destination_tags"]
        assert transport["source_branch_target"] == transport["destination_branch_target"] == base
        assert transport["source_ref_target"] == transport["destination_ref_target"] == tip
        # Retain typed, measured shell observations separately from Cairn-emitted receipts.
        transport_source = root / "output/transport.ncl"
        transport_source.write_text(TRANSPORT_CONTRACT + nickel(transport) + " | TransportObservation\n")
        (evidence / "transport.json").write_bytes(run([tools["nickel"], "export", "--format", "json", transport_source]))
        manifest = {
            "schema": "octet.cairn-cob-pilot.v1", "change": CHANGE, "workspace_id": WORKSPACE,
            "backend": BACKEND, "cairn_commit": args.cairn_commit, "base_git_oid": base,
            "actor": imported["actor"], "object_id": imported["object_id"],
            "import_operation_id": imported["operation_id"], "update_operation_id": tip,
            "final_content_root": updated["result_content_root"], "final_state_hash": updated["result_state_hash"],
            "namespace_ref": namespace,
            "bundle": {"path": bundle.name, "blake3_hex": digest(bundle.read_bytes()), "heads": heads},
            "transport": {key: transport[key] for key in ("source_ref_target", "destination_ref_target", "source_remotes", "destination_remotes", "artifact_tree_blake3")},
            "receipts": [{"name": name, "blake3_hex": digest((evidence / name).read_bytes())} for name in RECEIPTS],
            "required_non_claim_fragments": NON_CLAIMS,
        }
        manifest_source = root / "output/manifest.ncl"
        manifest_source.write_text(contract + nickel(manifest) + " | PilotManifest\n")
        (root / "output/manifest.json").write_bytes(run([tools["nickel"], "export", "--format", "json", manifest_source]))
        # A root-only base and precisely two COB operations; no other Git ancestry.
        commits = git("reload", "rev-list", "--all").splitlines()
        assert set(commits) == {base, imported["operation_id"], tip}, commits
        assert not git("reload", "show", "-s", "--format=%P", base)
        git("reload", "fsck", "--full", "--no-reflogs")
        for path in (root / "output").iterdir():
            if path.is_dir():
                shutil.copytree(path, fixture / path.name, dirs_exist_ok=True)
            else:
                shutil.copyfile(path, fixture / path.name)
        print(json.dumps({"base_git_oid": base, "object_id": imported["object_id"],
                          "update_operation_id": tip, "final_state_hash": updated["result_state_hash"],
                          "bundle_blake3": manifest["bundle"]["blake3_hex"], "commit_count": len(commits),
                          "tool_blake3": {key: run(["b3sum", "--no-names", value]).decode().strip() for key, value in tools.items()},
                          "source_remotes": transport["source_remotes"], "destination_remotes": transport["destination_remotes"]}, indent=2))


if __name__ == "__main__":
    main()
