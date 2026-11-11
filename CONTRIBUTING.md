# Contributing to Octet

Read the [contributor guide](docs/contributor-guide.md) for the source map,
Nix environment, and focused verification commands. Read the
[README](README.md) for supported entry points and evidence boundaries.

## Changes

- Open an issue for a bug or proposal at
  <https://github.com/OnixResearch/octet/issues>. Include the pinned Octet
  revision, platform, exact command, and a small reproducible example.
- Keep pull requests focused. Explain the consumer-visible behavior, the
  exercised checks, and any remaining limits. Update affected docs and the
  changelog; preserve generated-file and lifecycle conventions in the guide.
- Run Rust commands inside `nix develop`. Do not use private sibling checkouts,
  credentials, or local caches as evidence that a public checkout works.
- Do not include credentials, proprietary consumer source, raw private
  receipts, or machine-specific permissions. Use [SECURITY.md](SECURITY.md)
  rather than a public issue for exploitable security details.

## Contribution license

Submit original contributions under `AGPL-3.0-or-later`, the existing project
license in [LICENSE](LICENSE). You retain your copyright. Submit only material
you have the right to contribute, including any required employer permission.

For third-party material, preserve its original license and attribution,
identify the immutable upstream revision and local modifications, include the
required license texts, and update [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Do not present an upstream author's work as an original Octet contribution.
