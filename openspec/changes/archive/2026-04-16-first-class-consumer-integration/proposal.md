## Why

Tigerstyle downstream rollout works today, but it still depends on copied bash
scripts, hand-edited `default_scope` blocks, and consumer-local wrapper logic.
Aspen proved the approach can work, and the Seaglass check path exposed why the
current model is still too fragile: rollout behavior lives in a script copy
instead of a first-class product surface.

## What Changes

- **BREAKING**: replace the copied shell-wrapper workflow with a first-party
  consumer integration command exposed through a Rust CLI, and remove the old
  wrapper-specific test surface once equivalent CLI tests exist
- **BREAKING**: move consumer runner configuration out of script bodies and into
  canonical workspace metadata in `Cargo.toml`
- add first-class Nix integration so Nix consumers use a published flake app /
  check helper instead of copying shell glue
- require any repo-local helper (`just`, `cargo xtask`, `nix run`, CI entry
  point) to be a thin wrapper around the same engine and metadata instead of a
  second implementation
- keep lint tuning in consumer-owned `dylint.toml`, but separate that from the
  new runner metadata

## Capabilities

### New Capabilities
- `consumer-integration`: first-class downstream check execution, workspace
  metadata, Nix integration, and thin local entry points

### Modified Capabilities
- None

## Non-Goals

- preserving the copied shell-wrapper path as a supported active path
- preserving `TIGERSTYLE_PATH` / sibling-checkout discovery as the normal
  consumer contract
- moving lint thresholds, disabled lints, or lint levels out of `dylint.toml`

## Impact

- **Files**: new Rust CLI / library crates, flake outputs, rollout docs, and
  tests replacing the current template-script logic
- **APIs**: new `cargo tigerstyle ...` user-facing command and new
  `[workspace.metadata.tigerstyle]` consumer config contract
- **Dependencies**: no new product dependency required for consumers beyond the
  published CLI / flake path; internal implementation may reuse existing cargo /
  dylint plumbing
- **Systems**: downstream consumers like Aspen and Seaglass will stop copying
  tigerstyle shell scripts and instead call the first-class integration surface

## Verification

This change is done when a downstream workspace can run the first-class CLI and
its Nix equivalent from workspace metadata alone, without a copied tigerstyle
script, while preserving `Cargo.lock` and keeping helper entry points as thin
delegations.
