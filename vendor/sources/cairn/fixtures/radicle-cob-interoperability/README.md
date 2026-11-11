# Radicle COB interoperability fixture

This directory contains a deterministic signed Git COB bundle produced by the
standalone package under `producer/`. The producer imports no Cairn crate. It
writes exact Cairn action JSON through upstream
`radicle::storage::git::Repository` and direct `radicle-cob`, then exits. The
producer is fixture-generation evidence only: it is not a root-workspace member,
Nix input, released executable, runtime dependency, profile, node, or transport.

`provenance.ncl` is the typed source of truth. `provenance.json` is its
deterministic runtime export. Both the integration suite and the dedicated Nix
gate bind the bundle, producer source, standalone manifest, lockfile, pinned
library cohort, namespaced ref, object IDs, actors, and non-claims with BLAKE3.

## Deterministic regeneration

From the Cairn repository root:

```bash
nix develop -c cargo run --locked \
  --manifest-path fixtures/radicle-cob-interoperability/producer/Cargo.toml \
  -- /tmp/cairn-cob-reference-repository

git -C /tmp/cairn-cob-reference-repository bundle create \
  fixtures/radicle-cob-interoperability/external-cairn-cob.bundle --all

nix develop -c nickel export --format json \
  fixtures/radicle-cob-interoperability/provenance.ncl \
  > fixtures/radicle-cob-interoperability/provenance.json
```

Regeneration is intentionally manual and review-gated. Recompute and review the
BLAKE3 fields in `provenance.ncl` before exporting JSON. A changed bundle,
producer source, dependency lock, actor, ref, or object ID is evidence drift,
not an automatic fixture update.

The deterministic seed bytes are public test values. They are not secrets,
operator identities, or production authority.

## Claim boundary

Passing this fixture proves bounded interoperability between Cairn and the
pinned upstream storage/format cohort for the exercised Cairn type and linear
histories. It does not prove arbitrary Radicle-version compatibility,
publication, discovery, synchronization, convergence, remote trust, durability,
source correctness, or release eligibility.
