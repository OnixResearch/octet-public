# Default-Deny Gate Posture

Octet's checked path is opt-in per consumer: consumers must explicitly run
`cargo octet check`, install the pre-commit hook from
`templates/pre-commit-config.yaml`, or wire the gate into CI themselves. This
is the external reference style-gate posture inverted: external reference enforces `fmt` + `lint`
before every `build`, `check`, `validate`, and `eval --file` by default, with
bypass flags explicitly labeled "CI must not." Octet should offer the same
default-on posture for consumers who want it.

## What this provides

### 1. Cargo alias template

`templates/cargo-alias.toml` is a copyable `.cargo/config.toml` alias that
makes `cargo octet-check` run the Octet gate followed by `cargo check`:

```toml
[alias]
octet-check = "!cargo octet check -- && cargo check"
```

Consumers who want the default-on posture copy this into their
`.cargo/config.toml` and use `cargo octet-check` as their primary build
command. The leading `!` is required because Cargo shell aliases are the alias
form that can short-circuit with `&&`. The gate runs first; if it fails, the
build does not proceed.

### 2. Posture documentation

This document defines the posture:

- **Default-on**: the gate runs before the build, not after. The checked path
  is the primary path, not an opt-in.
- **Bypass is explicit and labeled**: `--allow-style-violations` and
  `OCTET_STYLE_GATE_DISABLE=1` exist for emergency local builds and
  one-off migrations. CI must not pass these flags.
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
  imperative-shell boundary. The cargo alias is the lighter-weight
  alternative.
- Enforcement that consumers adopt the posture. The templates are copyable;
  adoption is the consumer's decision. The posture doc makes the
  recommendation explicit.
- A claim that the gate proves correctness. The gate is operational gating
  (lints, fmt, standards), not formal proof. See
  `docs/trust-boundary-evidence.md` for the evidence tiers.
