## Why

Octet source acquisition, dependency trust receipts, Cairn content-addressed evidence, and Valence provenance adapters all need to describe where source facts came from: Git, local worktrees, vendored snapshots, registry packages, or generated artifacts. Octet already owns checked-path source acquisition for Rust code, making it the right repo to define the reusable source-boundary DTO core.

The extraction should share source identity, revision, dirty-state, vendoring, and non-claim primitives without moving filesystem traversal, dependency fetching, or trust decisions out of Octet shells and policy rails.

## What Changes

- Define source-boundary DTOs for Git, local, vendored, registry, generated, and unknown/manual-review source kinds.
- Add revision, dirty-state, selected-path, digest inventory, and source non-claim contracts.
- Add positive fixtures for current Octet cross-repo source acquisition records and dependency-trust examples.
- Add negative fixtures for missing revision metadata, dirty worktrees without policy, unbounded local paths, stale vendored snapshots, missing source non-claims, and overclaims.
- Keep filesystem walking, source fetching, Cargo metadata inspection, registry access, and artifact publication outside the pure core.

## Impact

- **Octet** owns the reusable source-boundary core for checked Rust evidence.
- **Valence/Cairn/Mantle** can consume source-boundary records as identity/linkage input.
- **Existing cross-repo-source-acquisition work** remains the Octet integration surface and can adopt this core once implemented.
