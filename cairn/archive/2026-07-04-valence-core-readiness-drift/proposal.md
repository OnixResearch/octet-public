## Why

`valence-core` now exposes more pure evidence surfaces than its package README and readiness rail describe. The seam is supposed to make future Valence extraction safer, but stale module inventories make reviewers guess which APIs are intentional and which Octet shell responsibilities are still retained. Before more extraction work lands, Octet needs a drift guard that keeps the crate README, crate-level docs, public module exports, and readiness rail aligned.

## What Changes

- Refresh the `valence-core` package/API documentation so every current public module is named with its boundary and non-claims.
- Strengthen `checks.${system}.valence-core-package-readiness` from metadata-shape proof into API-readiness proof that also exercises focused tests, docs, and package listing.
- Add deterministic assertions that the README, crate-level docs, and exported module inventory agree.
- Preserve the current no-publication/no-cutover boundary.

## Impact

- **Files**: `crates/valence-core/README.md`, `crates/valence-core/src/lib.rs`, `flake.nix`, `cairn/specs/valence-project-extraction/spec.md`.
- **Testing**: focused positive and negative `valence-core` readiness checks, `cargo test -p valence-core`, `cargo doc -p valence-core --no-deps`, `cargo package -p valence-core --allow-dirty --no-verify --list`, `nix build .#checks.x86_64-linux.valence-core-package-readiness --no-link -L`, and `nix run .#cairn -- validate --root .`.
