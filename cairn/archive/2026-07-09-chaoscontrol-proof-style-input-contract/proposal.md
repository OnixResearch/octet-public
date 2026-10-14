## Why

ChaosControl consumes Octet/Trellis proof-style and focused Tiger Style gates through pinned flake inputs and local checks. The coupling should be a declared contract: which Octet profile, Trellis primitive manifest, policy registry, and focused consumer crates are in scope.

## What Changes

- Add an Octet evidence contract for ChaosControl proof-style input rails.
- Record flake input identity, focused crate list, lint/profile identity, policy registry hash, and non-claims.
- Add positive and negative fixtures for current, stale, missing-profile, missing-crate, and overbroad proof-style claims.

## Impact

- ChaosControl can cite a stable Octet contract for proof-style gates.
- Octet keeps lint/profile ownership; ChaosControl keeps VMM/runtime semantics.
