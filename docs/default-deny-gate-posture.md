# Default-Deny Gate Posture

Octet's checked path is opt-in per consumer: consumers must explicitly run
`cargo octet check`, install the pre-commit hook from
`templates/pre-commit-config.yaml`, or wire the gate into CI themselves. This
is the external reference style-gate posture inverted: external reference enforces `fmt` + `lint`
before every `build`, `check`, `validate`, and `eval --file` by default, with
bypass flags explicitly labeled "CI must not." Octet should offer the same
default-on posture for consumers who want it.

## What this provides

### 1. Checked-build wrapper template

`templates/octet-check.sh` is a copyable POSIX shell wrapper that runs the
Octet gate followed by `cargo check`:

```sh
#!/bin/sh
set -eu

cargo octet check
exec cargo check "$@"
```

Consumers who want the default-on posture copy it into their workspace (for
example as `scripts/octet-check`), mark it executable, and use it as their
primary build command; arguments are passed to `cargo check` only. Cargo
aliases expand only to Cargo arguments and cannot run two commands or stop
between them, so the posture needs this wrapper. The gate runs first; if it
fails, the script exits with the gate's status and the build does not
proceed.

### 2. Posture documentation

This document defines the posture:

- **Default-on**: the gate runs before the build, not after. The checked path
  is the primary path, not an opt-in.
- **No hidden bypass**: Octet ships no flag or environment variable that
  skips the gate. Skipping it means running plain `cargo check` instead of
  the wrapper, which is visible in the command itself. CI must always run
  the gate.
- **Exit codes are stable for automation**: `0` clean, `1` lint findings,
  `2` integration failure. CI gates on non-zero.
- **The gate is byte-deterministic**: the same source + config + toolchain
  produce the same findings, sidecars, and provenance. No hidden entropy.

### 3. CI integration template

`templates/ci-gate.yml` is a copyable CI step that runs the gate in
isolation:

```yaml
- name: Octet checked path
  run: |
    cargo octet check --output-format sarif --artifact-dir artifacts/octet
    cargo octet artifact verify --artifact-dir artifacts/octet
```

This runs the gate, writes SARIF + sidecars, and verifies the artifact
directory before the build proceeds. CI should fail on any non-zero exit.

## What this does not provide

- A `build.rs` that automatically runs the gate on every `cargo build`. This
  would add latency to every incremental build and would couple the gate to
  the Rust build graph in a way that breaks the functional-core /
  imperative-shell boundary. The wrapper script is the lighter-weight
  alternative.
- A gate bypass flag or environment variable. None exists; the checked path
  is skipped only by not running it.
- Enforcement that consumers adopt the posture. The templates are copyable;
  adoption is the consumer's decision. The posture doc makes the
  recommendation explicit.
- A claim that the gate proves correctness. The gate is operational gating
  (lints, fmt, standards), not formal proof. See
  `docs/trust-boundary-evidence.md` for the evidence tiers.
