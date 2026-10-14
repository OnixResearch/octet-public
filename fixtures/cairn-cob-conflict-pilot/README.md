# Octet Cairn COB conflict-recovery fixture

This fixture records a complete local conflict/recovery run against independently rooted Cairn source revision `6b7e19531554714b6760e8cacc2a980e576d36af`. Its Git ancestry starts at the empty synthetic commit `b91fdcff9139492e55a7e64a142de6d0684ddb03`, not Octet or Cairn repository history. The real CLI was built offline/locked with pinned Cargo from the read-only source selected by Nix metadata; this is CLI execution evidence, not a claim that the Nix package build succeeded.

`manifest.ncl` is the typed source of truth and `manifest.json` is its reviewed Nickel evaluator export. The fixture manifest schema is `octet.cairn-cob-conflict-pilot.v2`: `synthetic_base_git_oid` replaces the historical Octet-base field without an alias. Real Cairn receipt schemas and the `radicle-cob.v1` backend are unchanged.

`evidence/` contains seventeen unmodified real Cairn stdout receipts and one **shell-measured** local Git transport observation. `transport.ncl` is the typed source for `evidence/transport.json`, exported by the same reviewed evaluator. Its supplemental non-claims describe the limits of the shell observations; they are not statements emitted by Cairn. No signed content or provider stdout was redacted or rewritten after generation.

## Observed result

- Creator-signed import with one immutable delegate: preview and execute `PASS`.
- Two independent tasks updates from that exact signed basis: creator `signed_creator` and delegate `embedded_delegate`, both preview/execute `PASS`.
- Inspection store: two competing tasks heads, metadata-only inspection and two exact-selector content inspections `PASS`.
- Ordinary update while conflicted: expected `FAIL`, blocked, preview-only, and not mutated; Git refs stayed unchanged.
- Explicit creator-head selection: preview/execute `PASS`; the resolution has exactly both competing operations as direct Git parents, preserving their actual order and the losing revision.
- Independent bundle reload: status and conflict inspection `PASS`, no remaining artifact conflict, and identical resolved state hash.
- Native checkout: named change absent before checkout and after preview; execute `PASS`, destination `current`, exact creator tasks bytes materialized.
- Final lifecycle condition: `stale`. Selecting a content head does not silently rewrite the earlier lifecycle subject root.

The bundle has exactly three heads: `refs/heads/main`, the creator's resolved namespace ref, and the delegate's losing namespace ref. Its complete graph contains five commits and exactly one root, whose tree is empty. Every temporary bare store had repository-local `cob-fixture@example.invalid` display metadata and no configured remote. `heads/` retains both exact competing `tasks.md` values (60 and 61 bytes).

Measured operation identities:

| Observation | Git object ID |
| --- | --- |
| Signed import root/object | `030a0e27d379e6e5ff67f6eac940e2369a8169b1` |
| Creator competing head | `bdac0c8ff7cc99c781d382d43e69b9a8e10dc1fe` |
| Delegate competing head, retained losing revision | `91ad968acd64ce5d4d26439e9ba72261c96e2653` |
| Resolution | `61922203bead59f454200971ffcd9e016f84840e` |

The recovery bundle BLAKE3 is `8c8475bc5745607104365c3b52dab656061595264ede305178a8848c95f937ef`. All eighteen evidence hashes, both content hashes and lengths, exact bundle heads, direct-parent order, policy binding, and state/content roots are measured in the typed manifest.

## Replay

From the candidate Octet root, use an already-built Cairn executable from the exact source revision above and the repository's `octet-cob-fixture-keys` executable. The key helper creates temporary creator/delegate keystores from **public fixture seeds 51 and 52**; these identities have no production authority. Set `CAIRN_COMMIT=6b7e19531554714b6760e8cacc2a980e576d36af`, `CAIRN_POLICY` to that source's generated policy, and `NICKEL` to the reviewed Nickel 1.17.0 executable:

```sh
python3 tools/replay-cairn-cob-conflict.py \
  --cairn "$CAIRN" \
  --keys "$OCTET_COB_FIXTURE_KEYS" \
  --policy "$CAIRN_POLICY" \
  --cairn-commit "$CAIRN_COMMIT" \
  --nickel "$NICKEL"
```

`--output DIRECTORY` selects a separate replay output. `TMPDIR` may select a filesystem with room for the temporary stores. The script requires `git` and `b3sum` on `PATH` and rejects evaluators outside the exact reviewed cohort (`nickel-lang-cli nickel 1.17.0 (rev 1320a98)`). The supplied source identity must actually belong to the executable; the script binds that identity but cannot infer an executable's source provenance.

The script creates fresh temporary bare stores and workspaces, ignores ambient Git configuration, configures no remotes, and performs only local object transport. Cairn runs from the scenario root with relative illustrative repository, keystore, workspace, and policy paths. The reload workspace's native `.cairn/changes` parent exists before checkout, but the named change remains absent until execute. Temporary private-key files and stores are removed after the run; no private-key files or original Git objects are retained in this fixture.

Operation timestamps may change replay bytes and hashes, so receipt, content, and bundle hashes are measured anew with `b3sum`; the typed manifest and shell-transport source are then exported by the reviewed evaluator. The replay stops on unexpected CLI refusal instead of inventing later observations.

## Validation and boundaries

The focused integration test preserves hash bindings, actor authority, divergent basis, both conflict values, refusal/no-mutation, complete resolution basis, reload, checkout, lifecycle, non-claims, and negative tampered-basis checks. The producer exercises real CLI behavior but does not build, format, lint, or run tests. Integration verification commands are:

```sh
nix develop -c cargo test --test cairn_cob_conflict_pilot
nix build .#checks.x86_64-linux.cairn-cob-conflict-pilot --no-link -L
```

This is bounded local signed Git COB integration evidence only. It does not prove semantic merge correctness, Radicle runtime participation, publication, remote authorization, availability, durability, synchronization, distributed convergence, source correctness, whole-program correctness, or release eligibility. No publication, API authentication, or network transport is performed or authorized by this replay.
