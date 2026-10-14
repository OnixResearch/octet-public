## Why

Octet runner and artifact-verification commands read source trees, cache directories, proof receipts, and artifact bundles supplied by operators. Those shell surfaces are good candidates for `cap-std` because they open an ambient root once and then operate on relative artifact paths.

## What Changes

- Adopt `cap-std` at Octet runner, cache, artifact-dir, proof-rail, and source-acquisition shell boundaries.
- Introduce typed capability roots around artifact, cache, proof, source, and workspace roots.
- Keep lint analysis cores, standards validators, and Valence DTO construction independent of capability filesystem APIs.
- Add positive and negative fixtures for valid artifact bundles, `../`, absolute paths, symlink escapes, stale/missing roots, and cache escape attempts.

## Impact

- Octet checked-path artifact authority becomes explicit and root-confined.
- Existing artifact formats, provenance hashes, and runner outputs remain stable for valid inputs.
- Invalid paths deny before artifact verification, cache access, or source scanning crosses declared roots.
