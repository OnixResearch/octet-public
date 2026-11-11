#!/usr/bin/env python3
"""Exercise real reviewed-source admission against temporary byte/path tampering."""

import argparse
import importlib.util
import json
import shutil
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("verify-reviewed-object-source.py")
spec = importlib.util.spec_from_file_location("reviewed_source_verifier", SCRIPT)
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def copy_reviewed_inputs(source: Path, destination: Path) -> None:
    reviewed = json.loads((source / verifier.REVIEWED_PATH).read_text())
    paths = {
        verifier.REVIEWED_PATH,
        verifier.CANDIDATE_PATH,
        verifier.APPROVAL_ARCHIVE_PATH,
        "cargo-octet/Cargo.toml",
        "Cargo.lock",
        "flake.nix",
        "flake.lock",
        "standards/valence-core-cutover/generated-manifest.json",
    }
    paths.update(row["path"] for row in reviewed["source_file_hashes"])
    paths.update(row["path"] for row in reviewed["reference_hashes"])
    paths.update(row["path"] for row in reviewed["reviewed_octet_modules"])
    for relative in paths:
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / relative, target)


def mutate_json(path: Path, change) -> None:
    value = json.loads(path.read_text())
    change(value)
    path.write_text(json.dumps(value, indent=2) + "\n")


def resurrect_historical_approval(repo: Path) -> None:
    old_path = repo / verifier.APPROVAL_MANIFEST_PATH
    old_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(repo / verifier.APPROVAL_ARCHIVE_PATH, old_path)


def reject_after(source: Path, module: Path, description: str, mutate, expected: str) -> None:
    with tempfile.TemporaryDirectory(prefix="octet-reviewed-source-probe-") as directory:
        repo = Path(directory)
        copy_reviewed_inputs(source, repo)
        mutate(repo)
        try:
            verifier.admit(repo, module)
        except verifier.AdmissionError as error:
            assert expected in str(error), f"{description} rejected for wrong reason: {error}"
        else:
            raise AssertionError(f"{description} unexpectedly admitted")
        print(f"reviewed-v2-source negative: {description} REJECTED")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--valence-module", type=Path, required=True)
    args = parser.parse_args()
    source = args.repo.resolve(strict=True)
    module = args.valence_module.resolve(strict=True)
    verifier.admit(source, module)
    print("reviewed-v2-source positive: approved bytes ADMITTED")
    with tempfile.TemporaryDirectory(prefix="octet-reviewed-path-probe-") as directory:
        root = Path(directory)
        (root / "selected.rs").write_text("fn source() {}\n")
        assert verifier.safe_file(root, "selected.rs") == root / "selected.rs"
        for unsafe in ("/etc/passwd", "../selected.rs", "nested/../../selected.rs", "nested//selected.rs", "./selected.rs", "nested\\selected.rs"):
            try:
                verifier.safe_file(root, unsafe)
            except verifier.AdmissionError as error:
                assert "unsafe path" in str(error), f"{unsafe}: {error}"
            else:
                raise AssertionError(f"unsafe selection admitted: {unsafe}")
    print("reviewed-v2-source traversal/absolute path controls: REJECTED")

    candidate_path = verifier.CANDIDATE_PATH
    reviewed_path = verifier.REVIEWED_PATH
    receipt = "cargo-octet/tests/fixtures/object-corpus/source-token-v2/positive-v2-receipt.json"
    source_file = "cargo-octet/tests/fixtures/object-corpus/source-token-v2/src/lib.rs"
    reject_after(source, module, "token-stable source byte edit", lambda repo: (repo / source_file).write_text((repo / source_file).read_text() + "\n// ordinary trivia\n"), "selected source byte drift")
    reject_after(source, module, "missing reference", lambda repo: (repo / receipt).unlink(), "missing path")
    reject_after(source, module, "reference byte edit", lambda repo: (repo / receipt).write_text((repo / receipt).read_text() + " "), "saved reference byte drift")
    reject_after(source, module, "false boundary", lambda repo: mutate_json(repo / receipt, lambda value: value["boundary"].update(claims_correctness=True)), "saved reference byte drift")
    reject_after(source, module, "missing archived approval", lambda repo: (repo / verifier.APPROVAL_ARCHIVE_PATH).unlink(), "missing path")
    reject_after(source, module, "altered archived approval", lambda repo: (repo / verifier.APPROVAL_ARCHIVE_PATH).write_text((repo / verifier.APPROVAL_ARCHIVE_PATH).read_text() + " "), "archived approval byte drift")
    reject_after(source, module, "resurrected active approval", resurrect_historical_approval, "historical active approval path was resurrected")
    reject_after(source, module, "PENDING reviewed status", lambda repo: mutate_json(repo / reviewed_path, lambda value: value.update(approval_status="PENDING")), "independently reviewed approval binding drift")
    reject_after(source, module, "rebound candidate", lambda repo: mutate_json(repo / candidate_path, lambda value: value.update(status="REVIEWED")), "independently reviewed candidate snapshot drift")
    reject_after(source, module, "unsorted reviewed selection", lambda repo: mutate_json(repo / reviewed_path, lambda value: value["source_selections"]["internal_corpus"].reverse()), "independently reviewed approval binding drift")
    reject_after(source, module, "absolute selection", lambda repo: mutate_json(repo / reviewed_path, lambda value: value["source_selections"]["fixed_quoted_url"].__setitem__(0, "/etc/passwd")), "independently reviewed approval binding drift")
    reject_after(source, module, "wrong Cargo revision", lambda repo: (repo / "cargo-octet/Cargo.toml").write_text((repo / "cargo-octet/Cargo.toml").read_text().replace(verifier.REVISION, "0" * 40)), "selected Cargo dependency drift")
    reject_after(source, module, "wrong Nix lock revision", lambda repo: mutate_json(repo / "flake.lock", lambda value: value["nodes"]["valence"]["locked"].update(rev="0" * 40)), "Nix locked Valence revision")
    reject_after(source, module, "symlink escaping source", lambda repo: ((repo / source_file).unlink(), (repo / source_file).symlink_to("/etc/passwd")), "symlink path")
    print("reviewed-v2-source tamper matrix: PASS")


if __name__ == "__main__":
    main()
