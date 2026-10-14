## Why

The May rename made Octet canonical, but active files still carry `slotcar` names in specs, docs, Nix outputs, Cargo package surfaces, tests, fixtures, and legacy fallback code. A blind search-and-replace would either corrupt archived evidence or silently change compatibility behavior. We need a deterministic Steel-driven sweep that can dry-run, apply, and prove the remaining current-tree identity surface.

## What Changes

- Add a repo-owned Steel sweep helper that tokenizes active text files, applies the `slotcar` -> `octet` / `Slotcar` -> `Octet` mapping, handles compound names such as `cargo-slotcar`, `slotcar-standards`, `[slotcar]`, `workspace.metadata.slotcar`, `slotcar::`, and `slotcar-allow`, and emits deterministic dry-run/apply/verify reports.
- Use that helper to replace active product/tool identity references across docs, main OpenSpec specs, templates, Cargo/Nix surfaces, standards code, fixtures, and tests.
- Retire `slotcar` compatibility aliases where the sweep can update callers mechanically; any retained legacy occurrence must be explicitly allowlisted with rationale and focused verification.
- Save evidence proving the Steel dry-run inventory, apply receipt, final zero-unapproved report, and smallest relevant Cargo/Nix/doc checks.

## Non-Goals

- Changing lint semantics, policy severity, or standards rule behavior.
- Rewriting historical transcripts under `openspec/changes/archive/**`, `cairn/archive/**`, or other archived evidence bundles except if a current spec points at a dead active path.
- Replacing `Tiger Style` philosophy language unless it is only being used as product identity.

## Capabilities

### New Capabilities

- `steel-identity-sweep`: maintainers can mechanically rename product identity strings with deterministic Steel dry-run/apply/verify reports.

### Modified Capabilities

- `octet-public-identity`: active docs/specs must contain no unapproved `slotcar` product identity after the sweep.
- `consumer-integration`: command, package, config, and receipt surfaces must be Octet-first, with any legacy `slotcar` compatibility retained only by explicit allowlist.
- `standards-codification`: standards package, policy, and artifact names must use Octet identity unless an archived or compatibility boundary is documented.

## Impact

- **Files / artifacts**: `tools/octet-identity-sweep.scm`, `README.md`, `CHANGELOG.md`, `docs/**`, `templates/**`, `openspec/config.yaml`, `openspec/specs/**`, `Cargo.toml`, `Cargo.lock`, `flake.nix`, `cargo-octet/**`, `crates/slotcar-standards/**` or renamed successor paths, `tests/**`, `ui_tests/**`, `standards/policies/**`, and `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/**`.
- **Concrete scope**: current non-archived repository surfaces and generated fixtures; archived evidence is scanned and classified but not rewritten.
- **Downstream validation boundaries**: no live downstream repo migration; retained external examples must point at Octet paths or be labeled archive/history.
- **Ordering / dependencies**: implement Steel dry-run first, review replacement/allowlist report, apply file/path renames, refresh generated artifacts, then run focused checks.
- **Verification / evidence**: Steel dry-run/apply/final reports with zero unapproved active `slotcar` rows; focused config/CLI/standards tests; `git diff --check`; relevant Nix package/check smoke when Nix surfaces change.
