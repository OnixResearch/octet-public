#!/usr/bin/env python3
"""Freeze Octet's real executable job and complete ordinary Nix tool graph.

Acquisition only: no build, proof, theorem admission or acceptance is performed.
The JSON derivation graph is Nix's existing format, not a new proof registry.
"""
import argparse
import json
import re
from pathlib import Path
import shutil
import subprocess

MAX_GRAPH_BYTES = 268_435_456
MAX_DERIVATIONS = 32_768
MAX_SOURCES = 16_384
MAX_SOURCE_BYTES = 34_359_738_368


def command(argv, maximum):
    result = subprocess.run([str(value) for value in argv], capture_output=True, timeout=180)
    if result.returncode:
        raise RuntimeError(result.stderr[:1_048_576].decode(errors="replace"))
    if len(result.stdout) > maximum or len(result.stderr) > 1_048_576:
        raise ValueError("producer metadata command exceeds output bound")
    return result.stdout


def source_bytes(path):
    if path.is_symlink():
        return len(str(path.readlink()).encode())
    if path.is_file():
        return path.stat().st_size
    if path.is_dir():
        total = 0
        for child in path.iterdir():
            total += source_bytes(child)
            if total > MAX_SOURCE_BYTES:
                raise ValueError("source acquisition byte bound exceeded")
        return total
    raise ValueError(f"unsupported source object: {path}")


def producer_graph(graph, drv, source_root):
    """Inspect raw legacy/v4 references only; never rewrite or rehash as legacy."""
    versioned = "version" in graph or "derivations" in graph
    if versioned:
        if type(graph.get("version")) is not int or graph["version"] != 4 or not isinstance(graph.get("derivations"), dict):
            raise ValueError("unsupported producer graph schema version")
        nodes = graph["derivations"]
        root_key = drv.removeprefix("/nix/store/")
        source_key = source_root.removeprefix("/nix/store/")
    else:
        nodes, root_key, source_key = graph, drv, source_root
    if not nodes or len(nodes) > MAX_DERIVATIONS or root_key not in nodes:
        raise ValueError("producer graph root missing or graph exceeds bound")

    def store_object(path):
        if not isinstance(path, str):
            raise ValueError("invalid producer store object identity")
        name = path if versioned else path.removeprefix("/nix/store/")
        if (not versioned and not path.startswith("/nix/store/")) or not re.fullmatch(r"[0-9abcdfghijklmnpqrsvwxyz]{32}-[A-Za-z0-9+._?=-]{1,222}", name):
            raise ValueError("invalid producer store object identity")
        return "/nix/store/" + name

    store_object(root_key)
    store_object(source_key)
    sources = set()
    for path, node in nodes.items():
        if not isinstance(node, dict):
            raise ValueError("malformed producer derivation node")
        store_object(path)
        if not path.endswith(".drv"):
            raise ValueError("invalid producer derivation identity")
        if versioned:
            if type(node.get("version")) is not int or node["version"] != 4 or "inputDrvs" in node or "inputSrcs" in node:
                raise ValueError("unsupported or mixed producer node schema version")
            dependencies, srcs = node["inputs"]["drvs"], node["inputs"]["srcs"]
        else:
            if "version" in node or "inputs" in node:
                raise ValueError("mixed legacy and versioned producer node schema")
            dependencies, srcs = node["inputDrvs"], node["inputSrcs"]
        if not isinstance(dependencies, dict) or not isinstance(srcs, list):
            raise ValueError("malformed producer input collections")
        for dependency, selected in dependencies.items():
            store_object(dependency)
            if dependency not in nodes:
                raise ValueError("incomplete recursive producer graph")
            if versioned and (not isinstance(selected, dict) or selected.get("dynamicOutputs") != {} or not isinstance(selected.get("outputs"), list) or not all(isinstance(value, str) for value in selected["outputs"])):
                raise ValueError("unsupported producer dynamic input or malformed output selection")
        sources.update(store_object(path) for path in srcs)
        if path == root_key and source_key not in srcs:
            raise ValueError("producer root does not bind immutable owner source")
    return sources, "nix-derivation-show-v4" if versioned else "nix-derivation-show-legacy"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--nix", type=Path, required=True)
    parser.add_argument("--nickel", type=Path, required=True)
    parser.add_argument("--flake", required=True, help="explicit immutable flake reference for this committed owner source")
    parser.add_argument("--revision", required=True, help="exact committed owner Git revision expected from the flake")
    parser.add_argument("--graph-format", required=True,
                        choices=["nix-derivation-show-legacy", "nix-derivation-show-v4"],
                        help="consumer's required raw Nix wire format; never convert or downgrade")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    nix, nickel = (path.resolve(strict=True) for path in (args.nix, args.nickel))
    if args.out.exists():
        parser.error("--out must not exist")
    if not re.fullmatch(r"[0-9a-f]{40}", args.revision):
        raise ValueError("--revision requires an exact immutable Git object ID")
    metadata = command([nix, "flake", "metadata", "--json", "--no-write-lock-file", args.flake], 1_048_576)
    meta = json.loads(metadata)
    locked_revision = meta.get("revision")
    if locked_revision != args.revision:
        raise ValueError("immutable flake revision differs from requested owner revision")
    locked_root = Path(meta["path"])
    for required in ["nix/mantle-aeneas-job.nix", "standards/proof-rails/aeneas-lean/mantle-job.ncl",
                     "tools/mantle-producer/export.py"]:
        if not (locked_root / required).is_file():
            raise ValueError(f"immutable owner source lacks {required}; recorded fixture pins cannot substitute")
    if (locked_root / "tools/mantle-producer/export.py").read_bytes() != Path(__file__).read_bytes():
        raise ValueError("exporter differs from selected immutable owner source")
    policy = locked_root / "standards/proof-rails/aeneas-lean/mantle-job.ncl"
    job_bytes = command([nickel, "export", "--format", "json", policy], 1_048_576)
    job = json.loads(job_bytes)
    if job["schema_version"] != "octet-mantle-producer-job/v1" or job["owner"] != "Octet":
        raise ValueError("unsupported owner job contract")
    attribute = job["package_attribute"]
    # Nix's locked URL includes the resolved revision, even if the input was a branch.
    target = f"{meta['url']}#{attribute}"
    drv = command([nix, "eval", "--raw", "--no-write-lock-file", target + ".drvPath"], 4096).decode().strip()
    graph_bytes = command([nix, "derivation", "show", "--recursive", drv], MAX_GRAPH_BYTES)
    graph = json.loads(graph_bytes)
    if not isinstance(graph, dict) or not graph or len(graph_bytes) > MAX_GRAPH_BYTES:
        raise ValueError("invalid or oversized recursive producer graph")
    sources, graph_format = producer_graph(graph, drv, str(locked_root))
    if graph_format != args.graph_format or graph_format not in job["tools"]["graph_formats"]:
        raise ValueError("exported graph format differs from explicit consumer contract")
    if len(sources) > MAX_SOURCES:
        raise ValueError("source count bound exceeded")
    total = 0
    for source in sorted(sources):
        path = Path(source)
        if path.parent != Path("/nix/store"):
            raise ValueError("undeclared non-store source input")
        total += source_bytes(path)
        if total > MAX_SOURCE_BYTES:
            raise ValueError("source acquisition aggregate byte bound exceeded")
    args.out.mkdir(parents=True)
    (args.out / "job.json").write_bytes(job_bytes)
    (args.out / "tool-closure.derivation-json.json").write_bytes(graph_bytes)
    (args.out / "flake-metadata.json").write_bytes(metadata)
    (args.out / "flake.lock").write_bytes((locked_root / "flake.lock").read_bytes())
    (args.out / "root-derivation").write_text(drv + "\n")
    (args.out / "source-revision").write_text(locked_revision + "\n")
    source_root = args.out / "source-payloads"
    source_root.mkdir()
    for source in sorted(sources):
        path = Path(source)
        destination = source_root / path.name
        if path.is_dir() and not path.is_symlink():
            shutil.copytree(path, destination, symlinks=True)
        elif path.is_symlink():
            destination.symlink_to(path.readlink())
        else:
            shutil.copy2(path, destination)
    print(json.dumps({"root_derivation": drv, "source_revision": locked_revision,
                      "graph_format": graph_format, "derivations": len(graph["derivations"]) if graph_format == "nix-derivation-show-v4" else len(graph),
                      "source_objects": len(sources),
                      "source_bytes": total, "proof_executed": False}, indent=2))


if __name__ == "__main__":
    main()
