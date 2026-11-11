#!/usr/bin/env python3
"""Operator-only real-byte smoke; supply built, pinned producer executables."""
import pathlib
import subprocess
import sys
import tempfile


def run(argv, success=True):
    result = subprocess.run(argv, capture_output=True, text=True, check=False)
    if (result.returncode == 0) != success:
        raise RuntimeError(f"unexpected exit {result.returncode}: {argv}\n{result.stdout}\n{result.stderr}")
    return result


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: smoke.py /absolute/cargo-octet /absolute/wasm-tools")
    producer, wasm = map(str, map(pathlib.Path, sys.argv[1:]))
    if not all(pathlib.Path(path).is_absolute() for path in (producer, wasm)):
        raise SystemExit("producer executable paths must be absolute")
    root = pathlib.Path(__file__).resolve().parent
    config = root.parent / "generated/profiles.json"
    source = (root / "fixtures/positive/molten-abi.wat").read_text()
    cases = [
        ("admitted", source, True),
        ("unbounded", source.replace("1 2)", "1)"), False),
        ("wrong-host", source.replace('"assert"', '"clock"'), False),
        ("wrong-alloc", source.replace('"molten_alloc"', '"not_alloc"'), False),
        ("wasi", source.replace('"molten:hostcall"', '"wasi_snapshot_preview1"'), False),
        ("component", "(component)", False),
        ("proposal", source.replace("global.get $next local.tee $ptr", "global.get $next i32.extend8_s local.tee $ptr"), False),
    ]
    with tempfile.TemporaryDirectory(prefix="octet-spacewasm-smoke-") as directory:
        base = pathlib.Path(directory)
        for name, text, admitted in cases:
            wat = base / f"{name}.wat"
            module = base / f"{name}.wasm"
            receipt = base / name
            wat.write_text(text)
            run([wasm, "parse", str(wat), "-o", str(module)])
            run([producer, "evidence", "collect", "--rail", "wasm-artifact", "--input", str(module),
                 "--config", str(config), "--profile", "spacewasm-molten-abi-pilot",
                 "--artifact-dir", str(receipt)], admitted)
            if admitted:
                run([producer, "artifact", "verify", "--artifact-dir", str(receipt)])
                artifact = receipt / "wasm-artifact.bin"
                artifact.write_bytes(artifact.read_bytes() + b"changed-exact-bytes")
                run([producer, "artifact", "verify", "--artifact-dir", str(receipt)], False)
            print(f"{name}: expected {'admission plus tamper denial' if admitted else 'denial'} observed")


if __name__ == "__main__":
    main()
