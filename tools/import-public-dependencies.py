#!/usr/bin/env python3
"""Import/check Octet's exact, licensed dependency distributions; never build products."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "vendor/dependencies.ncl"
PROJECTION = ROOT / "vendor/dependencies.json"
SUMS = ROOT / "vendor/SHA256SUMS"


def run(args, **kwargs):
    return subprocess.run(args, check=True, stdout=subprocess.PIPE, **kwargs).stdout


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def inventory(directory):
    return {p.relative_to(ROOT).as_posix(): sha256(p.read_bytes())
            for p in sorted(directory.rglob("*")) if p.is_file()}


def check_sources(policy):
    expected = dict(line.rstrip("\n").split("  ", 1)[::-1]
                    for line in SUMS.read_text().splitlines())
    actual = {}
    for source in policy["sources"]:
        actual.update(inventory(ROOT / source["destination"]))
    actual.update(inventory(ROOT / "vendor/cargo"))
    if actual != expected:
        changed = sorted(path for path in actual.keys() | expected.keys()
                         if actual.get(path) != expected.get(path))
        raise ValueError("dependency distribution drift: " + ", ".join(changed))
    for source in policy["sources"]:
        for package in source["cargo_packages"]:
            directory = ROOT / "vendor/cargo" / package
            checksums = json.loads((directory / ".cargo-checksum.json").read_text())
            for path, digest in checksums["files"].items():
                if sha256((directory / path).read_bytes()) != digest:
                    raise ValueError("Cargo checksum drift: " + package + "/" + path)
    print("dependency distributions: exact recorded files and Cargo checksums verified")


def archive_source(source, repository):
    revision = source["revision"]
    if run(["git", "-C", str(repository), "rev-parse", revision + "^{commit}"]).decode().strip() != revision:
        raise ValueError("source is not the exact admitted commit: " + source["name"])
    archive = run(["git", "-C", str(repository), "archive", "--format=tar", revision,
                   "--", *source["paths"]])
    if sha256(archive) != source["archive_sha256"]:
        raise ValueError("upstream archive differs from the admitted distribution: " + source["name"])
    return archive


def sync(policy, source_paths, cargo):
    names = {source["name"] for source in policy["sources"]}
    if set(source_paths) != names:
        raise ValueError("--sync requires exactly one --source NAME=CHECKOUT for: " + ", ".join(sorted(names)))
    # Validate every immutable input before changing any distribution file.
    archives = {source["name"]: archive_source(source, source_paths[source["name"]])
                for source in policy["sources"]}
    with tempfile.TemporaryDirectory(prefix="octet-dependency-import-") as temporary:
        temporary = Path(temporary)
        env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL="/dev/null",
                   GIT_TERMINAL_PROMPT="0", GIT_ASKPASS="false", SSH_ASKPASS="false",
                   CARGO_HOME=str(temporary / "cargo-home"))
        # Rewrites are process-scoped intake inputs, never product configuration.
        env["GIT_CONFIG_COUNT"] = str(len(policy["sources"]))
        for index, source in enumerate(policy["sources"]):
            env[f"GIT_CONFIG_KEY_{index}"] = "url." + source_paths[source["name"]].as_uri() + ".insteadOf"
            env[f"GIT_CONFIG_VALUE_{index}"] = source["repository"]
        cargo_output = temporary / "cargo-vendor"
        run([cargo, "vendor", "--locked", "--versioned-dirs", str(cargo_output)], cwd=ROOT, env=env)
        selected = temporary / "selected"
        selected.mkdir()
        for source in policy["sources"]:
            for package in source["cargo_packages"]:
                package_source = cargo_output / package
                if not package_source.is_dir():
                    raise ValueError("Cargo did not vendor the admitted package: " + package)
                shutil.copytree(package_source, selected / package)
                # Upstream workspace licenses are not always inside Cargo's package list.
                upstream = ROOT / source["destination"]
                with tarfile.open(fileobj=io.BytesIO(archives[source["name"]])) as archive:
                    notices = [m for m in archive.getmembers() if m.isfile()
                               and (m.name.startswith("LICENSES/") or "/" not in m.name)
                               and (m.name.startswith("LICENSE") or m.name == "NOTICE")]
                    for notice in notices:
                        destination = selected / package / notice.name
                        destination.parent.mkdir(parents=True, exist_ok=True)
                        destination.write_bytes(archive.extractfile(notice).read())
                directory = selected / package
                checksum_file = directory / ".cargo-checksum.json"
                checksums = json.loads(checksum_file.read_text())
                checksums["files"] = {p.relative_to(directory).as_posix(): sha256(p.read_bytes())
                                      for p in sorted(directory.rglob("*"))
                                      if p.is_file() and p != checksum_file}
                checksum_file.write_text(json.dumps(checksums, sort_keys=True, separators=(",", ":")))
        for source in policy["sources"]:
            destination = ROOT / source["destination"]
            if destination.exists():
                shutil.rmtree(destination)
            destination.mkdir(parents=True)
            with tarfile.open(fileobj=io.BytesIO(archives[source["name"]])) as archive:
                archive.extractall(destination, filter="data")
        destination = ROOT / "vendor/cargo"
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(selected, destination)
    hashes = {}
    for source in policy["sources"]:
        hashes.update(inventory(ROOT / source["destination"]))
    hashes.update(inventory(ROOT / "vendor/cargo"))
    SUMS.write_text("".join(digest + "  " + path + "\n" for path, digest in sorted(hashes.items())))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sync", action="store_true", help="import exact source objects and Cargo-normalized distributions")
    parser.add_argument("--source", action="append", default=[], metavar="NAME=CHECKOUT")
    parser.add_argument("--nickel", default="nickel")
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    exported = run([args.nickel, "export", "--format", "json", str(POLICY)])
    policy = json.loads(exported)
    source_paths = {}
    for argument in args.source:
        name, path = argument.split("=", 1)
        if name in source_paths:
            raise ValueError("duplicate source: " + name)
        source_paths[name] = Path(path).resolve()
    if args.sync:
        sync(policy, source_paths, args.cargo)
        PROJECTION.write_bytes(exported)
    elif PROJECTION.read_bytes() != exported:
        raise ValueError("vendor/dependencies.json is stale; regenerate from Nickel")
    check_sources(policy)


if __name__ == "__main__":
    main()
