## Why

Several stack repos depend on private `ssh://git@github.com/OnixResearch/...` sources or local sibling paths during development. Those sources can be appropriate, but the dependency boundary is often implicit: a lockfile records bytes while reviewers still need to know whether a source is public, private, local-only, credential-bound, or release-eligible.

Octet already owns dependency trust receipts and optional external evidence rails. This change adds cross-repo source-acquisition receipts so consumers can validate the source path policy behind stack-owned dependencies.

## What Changes

- Add a source-acquisition receipt family for stack-owned Git, local path, vendored, and registry dependencies.
- Record source URL or repo label, revision, lockfile identity, credential boundary, local-override policy, release eligibility, and non-claims.
- Add positive fixtures for public HTTPS/tag, approved private SSH, vendored Trellis snapshot, and local development override cases.
- Add negative fixtures for unpinned Git, path-only release dependencies, missing credential boundary, stale lock identity, and hidden sibling assumptions.

## Impact

- **Octet** can report source-acquisition posture separately from vulnerability/license and cargo-vet trust evidence.
- **Basalt/Aspen/ChaosControl/Trellis** can cite receipts for private or local stack inputs.
- **Release reviews** can distinguish reproducible source identity from credential-bound local development.
- **Testing** adds fixture-based receipt validation and artifact verification coverage.
