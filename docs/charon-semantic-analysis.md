# Charon semantic analysis rail

Octet's Charon semantic rail is an opt-in, recorded-only evidence path for one exact Rust item. It reuses the pinned Charon executable, compatibility identity, Bubblewrap network sandbox, and bounded executor from the Aeneas/Lean provider. It does not add a second Charon pin and is not part of starter `cargo octet check`.

## Run the reviewed profile

```sh
cargo octet evidence collect \
  --rail charon-semantic \
  --run \
  --workspace-root . \
  --config standards/charon-semantic/generated/profile.json \
  --profile octet-charon-semantic-safe-add-v1 \
  --external-report standards/charon-semantic/generated/comparison-parent.json \
  --artifact-dir artifacts/charon-semantic

cargo octet artifact verify \
  --artifact-dir artifacts/charon-semantic \
  --format json
```

The execution shell requires `OCTET_AENEAS_CHARON` and `OCTET_AENEAS_NETWORK_SANDBOX` to be absolute executable paths from the packaged provider closure. The Nix check configures these paths. Arbitrary `--command` input is rejected.

For deterministic offline replay without invoking Charon, replace `--run` with `--input PATH` to a recorded LLBC JSON snapshot. Import mode records no executed command.

## Closed extraction contract

`standards/charon-semantic/profile.ncl` binds:

- Charon revision `cb50ff16b9f1066b8a97dc06da704de2da2fa41c` and the existing Aeneas provider/compatibility identities;
- the `real_fixture` crate, `src/lib.rs`, and exact item `real_fixture::safe_add`;
- the reviewed `charon cargo --preset aeneas --dest-file snapshot.llbc --start-from real_fixture::safe_add` invocation;
- preserved borrow checking, x86_64 target metadata, environment allowlist, timeout, log, snapshot, item, and edge bounds;
- closed fact and effect-candidate vocabularies; and
- the `recorded_only` verification role and explicit non-claims.

The imperative shell copies only the selected fixture into a fresh work root, denies network access, bounds execution and logs, reads the declared snapshot, and removes the work root. The pure core parses the in-memory snapshot and rejects partial/errorful extraction, skipped borrow checking, source or target drift, broad/duplicate item selection, missing call endpoints, unsupported vocabularies, and malformed parent evidence.

## Artifact roles

A valid bundle contains distinct BLAKE3-bound roles:

| Artifact | Role | Authority |
|---|---|---|
| `charon-snapshot.llbc` | Raw pinned-provider output | Recorded transcript input |
| `charon-semantic-projection.json` | Stable IDs, source spans, target, calls, traits, control flow | Recorded static projection |
| `charon-semantic-analysis.json` | Conservative effect candidates, unknowns, and call cycles | Recorded manual-review evidence |
| `charon-semantic-comparison.json` | Agreement/disagreement with independent parent evidence | Recorded comparison without authority merge |
| `evidence-rail-receipt.json` | Profile, provider, role, limits, findings, and artifact identities | Transcript-assertion receipt |
| `provenance.jsonl` | Exact parent and artifact links | Replayable provenance |

The artifact verifier recomputes every file hash, checks exact provenance coverage, checks transcript-only tiers, and binds receipt roles back to artifact bytes.

## Source/token comparison

`standards/charon-semantic/comparison/legacy-source-token-receipt.ncl` models an independent legacy source-token analysis receipt. Its generated receipt identity is configured into `comparison-parent.ncl`; the Nix check recomputes that parent identity before accepting the adapter. Comparison requires the same source identity and records `agree`, `disagree`, `charon-only`, `source-token-only`, or `unknown` per fact. It never overwrites either parent and always sets `authority_merged` to `false`.

## Downstream boundary

Valence may ingest the typed receipt and role-separated artifacts as external evidence. Kamacite may transport their exact identities. Neither layer becomes a compiler-IR or semantic-analysis authority. Downstream consumers must preserve the transcript evidence tier and all non-claims; they may not reinterpret this bundle as `formal_proof`, property evidence, complete closure, semantic equivalence, or release readiness.

## Recorded references

The profile records two references without vendoring or calling them:

- [`mechanized-llbc`](https://github.com/AeneasVerif/mechanized-llbc) at `8a9b8bb3823ceceb47026837d23baea0934dfb27`, Apache-2.0, as a semantic-foundation review corpus.
- [`charon-rudra`](https://github.com/AeneasVerif/charon-rudra) at `0923bf7f21d4eb234d14bce86afc9690d0a4d00e`, Apache-2.0, as historical analysis-over-Charon design evidence.

These references transfer no proof, implementation, or analysis authority.
