## Why

Octet's function-address dependency identity is a source-token set hash, not a resolved Rust call graph or complete dependency closure. The smoke path preserved this identity as dependency evidence, so Octet should make the boundary explicit before downstream layers harden receipt schemas.

## What Changes

- Rename or qualify dependency identity fields and docs as token/source-shape dependency sets.
- Preserve compatibility for existing `dependency_set_hash` fields while preventing closure overclaims.
- Add positive and negative fixtures for dependency identity kind and overclaim text.

## Impact

Downstream Kamacite, Valence, Mantle, and Cairn can distinguish recorded token dependency identity from semantic call-graph closure.
