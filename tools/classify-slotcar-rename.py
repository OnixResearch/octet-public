#!/usr/bin/env python3
"""Classify remaining legacy-name strings during the Octet rename."""
from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_PARTS = {".git", ".direnv", "target", ".pi", ".agent"}
BINARY_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".pdf",
    ".wasm",
    ".rlib",
    ".so",
    ".dylib",
    ".a",
}
LEGACY_TERM_RE = re.compile(r"Tiger Style|tigerstyle|TIGERSTYLE|Slotcar|slotcar")
CATEGORIES = ("philosophy", "compatibility", "namespace", "archived", "needs-fix")


@dataclass(frozen=True)
class Occurrence:
    category: str
    term: str
    rel: str
    line_number: int
    text: str


def is_archived(rel: str) -> bool:
    return rel.startswith("openspec/changes/archive/")


def is_namespace(term: str, line: str) -> bool:
    lower = line.lower()
    namespace_tokens = (
        "tigerstyle::",
        "slotcar::",
        "register_tool(tigerstyle)",
        "register_tool(slotcar)",
        "#[warn(tigerstyle",
        "#[warn(slotcar",
        "#[deny(tigerstyle",
        "#[deny(slotcar",
        "#[allow(tigerstyle",
        "#[allow(slotcar",
        "-d tigerstyle::",
        "-d slotcar::",
        "-w tigerstyle::",
        "-w slotcar::",
        "libtigerstyle",
        "tigerstyle dylint",
        "tigerstyle lint",
        "source attr",
        "source-attr",
        "tool-scoped",
        "tool lint",
        "rust attr",
        "rustc lint",
    )
    return term.lower() in {"tigerstyle", "slotcar"} and any(token in lower for token in namespace_tokens)


def is_philosophy(term: str, line: str) -> bool:
    lower = line.lower()
    return (
        term == "Tiger Style"
        or "tigerbeetle" in lower
        or "coding philosophy" in lower
        or "coding discipline" in lower
        or "guide:" in lower
        or "see tiger style guide" in lower
        or "tiger style (" in lower
    )


def is_compatibility_context(rel: str, line: str) -> bool:
    lower = line.lower()
    rel_lower = rel.lower()

    if rel == "AGENTS.md" or rel.startswith(".agents/skills/"):
        return True
    if rel.startswith("openspec/specs/"):
        return True
    if rel.startswith("openspec/changes/rename-slotcar-to-octet/"):
        return True
    if rel.startswith("standards/policies/"):
        return True
    if rel.startswith("ui_tests/") and "dylint.toml" in rel:
        return True
    if "global `tigerstyle` skill" in line:
        return True
    if "/path/to/git/slotcar" in line or "/path/to/slotcar" in line:
        return True
    if line.startswith("TOOL=") and "/slotcar/" in line:
        return True
    if rel in {
        "flake.nix",
        "openspec/config.yaml",
        "src/config.rs",
        "src/structure/unjustified_allow.rs",
        "cargo-octet/src/cli.rs",
        "cargo-octet/src/config.rs",
        "cargo-octet/tests/unison_roadmap.rs",
        "templates/dylint.toml",
        "tests/consumer_integration.rs",
    }:
        return True
    if rel == "tools/classify-slotcar-rename.py":
        return True
    if "slotcar" in rel_lower or "tigerstyle" in rel_lower:
        return True

    compatibility_tokens = (
        "compatibility",
        "compatible",
        "legacy",
        "alias",
        "transition",
        "historical",
        "history",
        "migration",
        "fallback",
        "fallbacks",
        "precedence",
        "old command",
        "previous command",
        "retained",
        "route to the same engine",
        "wrapper",
        "env prefix",
        "environment variable",
        "bootstrap",
        "dylint",
        "policy id",
        "policy-id",
        "schema id",
        "static check",
        "standards check",
        "repo-local helper",
        "local checkout",
        "checkout exists",
        "current slotcar contract",
        "slotcar defaults",
        "enabled warn-by-default slotcar lints",
        "differs from slotcar",
        "slotcar fingerprint check",
        "slotcar/rust-toolchain.toml",
        "baking aspen's patch revision into `slotcar`",
        "slotcar: add phase-1 rollout path",
        "`slotcar` defaults",
        "slotcar.devtools.",
        "todo(slotcar)",
        "fixture",
        "snapshot",
        "test fixture",
        "cargo slotcar",
        "cargo-slotcar",
        "cargo tigerstyle",
        "cargo-tigerstyle",
        "slotcar-standards",
        "tigerstyle-standards",
        "[slotcar]",
        "[tigerstyle]",
        "workspace.metadata.slotcar",
        "workspace.metadata.tigerstyle",
        "tigerstyle_",
        "_tigerstyle",
        "tigerstyle-",
        "-tigerstyle",
        "slotcar_",
        "_slotcar",
        "slotcar-",
        "-slotcar",
        "tigerstyle = octet",
        "slotcar = octet",
        "tigerstyle = slotcar",
        "slotcar checkout",
        "rename to slotcar",
        "rename-slotcar-to-octet",
        "classify-slotcar-rename",
        "remaining-name classification",
    )
    return any(token in lower for token in compatibility_tokens)


def classify(term: str, rel: str, line: str) -> str:
    if is_archived(rel):
        return "archived"
    if is_namespace(term, line):
        return "namespace"
    if is_philosophy(term, line):
        return "philosophy"
    if is_compatibility_context(rel, line):
        return "compatibility"
    return "needs-fix"


def iter_text_files() -> list[Path]:
    paths: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        rel_path = path.relative_to(ROOT)
        rel = rel_path.as_posix()
        if any(part in SKIP_PARTS for part in rel_path.parts):
            continue
        if rel.endswith("/evidence/name-classification.txt"):
            continue
        if path.suffix.lower() in BINARY_SUFFIXES:
            continue
        paths.append(path)
    return sorted(paths, key=lambda p: p.relative_to(ROOT).as_posix())


def iter_occurrences() -> list[Occurrence]:
    rows: list[Occurrence] = []
    for path in iter_text_files():
        rel = path.relative_to(ROOT).as_posix()
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        for line_number, line in enumerate(text.splitlines(), start=1):
            for match in LEGACY_TERM_RE.finditer(line):
                term = match.group(0)
                rows.append(Occurrence(classify(term, rel, line), term, rel, line_number, line.strip()))
    return rows


def main() -> int:
    rows = iter_occurrences()
    counts = {category: 0 for category in CATEGORIES}
    for row in rows:
        counts[row.category] += 1

    print("Octet rename remaining-name classification")
    print(f"root: {ROOT}")
    print("terms: slotcar, Slotcar, tigerstyle, TIGERSTYLE, Tiger Style")
    for category in CATEGORIES:
        print(f"count.{category}: {counts[category]}")
    print("columns: category<TAB>term<TAB>path<TAB>line<TAB>text")
    for row in rows:
        print(f"{row.category}\t{row.term}\t{row.rel}\t{row.line_number}\t{row.text}")

    return 1 if counts["needs-fix"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
