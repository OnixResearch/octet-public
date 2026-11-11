from pathlib import Path

README_PATH = Path("README.md")
GUIDE_PATH = Path("docs/consumer-guide.md")

README_NEEDLES = [
    "cargo tigerstyle check --output-format sarif --artifact-dir artifacts/tigerstyle",
    "- `0` clean or warning-only runs",
    "- `1` error-level lint findings",
    "- `2` integration or cargo-check failures",
]

GUIDE_NEEDLES = [
    "cargo run --manifest-path /path/to/tigerstyle/cargo-tigerstyle/Cargo.toml -- check",
    "cargo tigerstyle check --output-format sarif --artifact-dir artifacts/tigerstyle",
    "When `--artifact-dir` is set, the runner writes `command.txt`, `status.json`, `summary.txt`, and the selected structured-output mirror (`results.jsonl` or `results.sarif`).",
    "- `0` clean or warning-only runs",
    "- `1` error-level lint findings",
    "- `2` integration/configuration failures, including underlying `cargo check` failures",
]


def require_all(path: Path, needles: list[str]) -> None:
    text = path.read_text()
    for needle in needles:
        if needle not in text:
            raise SystemExit(f"{path}: missing {needle!r}")
    print(f"{path}: ok")


require_all(README_PATH, README_NEEDLES)
require_all(GUIDE_PATH, GUIDE_NEEDLES)
print("docs contract: passed")
