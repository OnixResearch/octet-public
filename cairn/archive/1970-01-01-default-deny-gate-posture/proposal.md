## Why

Octet's checked path is opt-in per consumer: consumers must explicitly run
`cargo octet check`, install the pre-commit hook, or wire the gate into CI.
external reference enforces fmt+lint before every build by default, with bypass labeled
"CI must not." Octet should offer the same default-on posture for consumers
who want the checked path as their primary path, not an opt-in.

## What Changes

- Add `docs/default-deny-gate-posture.md`: posture documentation defining
  default-on, bypass-labeled, stable-exit-code, and byte-deterministic
  guarantees.
- Add `templates/cargo-alias.toml`: copyable `.cargo/config.toml` alias that
  makes `cargo octet-check` run the gate followed by `cargo check`.
- Add `templates/ci-gate.yml`: copyable CI step that runs the gate, writes
  SARIF + sidecars, and verifies the artifact directory.

## Impact

- **Files**: `docs/default-deny-gate-posture.md` (new),
  `templates/cargo-alias.toml` (new), `templates/ci-gate.yml` (new).
- **Testing**: documentation/template-only change; no code or tests modified.
- **Non-claims**: the gate is operational gating (lints, fmt, standards), not
  formal proof. The posture doc makes the recommendation explicit but does not
  enforce adoption.
