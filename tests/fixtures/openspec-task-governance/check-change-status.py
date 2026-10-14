#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

GOVERNED_UNCHECKED = re.compile(r"- \[ \] ((?:I|V|H)\d+)\b")
STATUS_PREFIX = "- Status: "
OPEN_PREFIX = "- Open tasks: "


def findings_for(tasks_path: Path) -> list[str]:
    lines = tasks_path.read_text().splitlines()
    try:
        verification_idx = lines.index("## Verification Coverage")
    except ValueError:
        return []

    status_indices = [i for i, line in enumerate(lines) if line == "## Change Status"]
    if not status_indices:
        return ["tasks.md is missing ## Change Status"]
    if len(status_indices) != 1:
        return ["Change status section is malformed or misplaced"]

    status_idx = status_indices[0]
    if status_idx != verification_idx - 3:
        return ["Change status section is malformed or misplaced"]

    try:
        status_line = lines[status_idx + 1]
        open_line = lines[status_idx + 2]
    except IndexError:
        return ["Change status section is malformed or misplaced"]

    if not status_line.startswith(STATUS_PREFIX) or not open_line.startswith(OPEN_PREFIX):
        return ["Change status section is malformed or misplaced"]

    status = status_line[len(STATUS_PREFIX) :]
    unchecked = [m.group(1) for line in lines[:status_idx] if (m := GOVERNED_UNCHECKED.match(line))]

    if status == "complete":
        if unchecked:
            return ["Change status says complete but unchecked tasks remain"]
        if open_line != f"{OPEN_PREFIX}none":
            return ["Change status section is malformed or misplaced"]
        return []

    if status == "in-progress":
        if not unchecked:
            return ["Change status says in-progress but no open tasks remain"]
        expected = f"{OPEN_PREFIX}{', '.join(unchecked)}"
        if open_line != expected:
            return ["Change status open-task list does not match unchecked tasks"]
        return []

    return ["Change status section is malformed or misplaced"]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-change-status.py <tasks.md>", file=sys.stderr)
        return 2
    tasks_path = Path(sys.argv[1])
    for finding in findings_for(tasks_path):
        print(finding)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
