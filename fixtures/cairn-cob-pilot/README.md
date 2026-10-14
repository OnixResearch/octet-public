# Octet Cairn COB pilot fixture

This fixture is a fresh, bounded local observation from independently rooted Cairn source commit `6b7e19531554714b6760e8cacc2a980e576d36af`, also recorded in `manifest.ncl` (`cairn_commit`). It replaces a historical downstream bundle: **no original Octet or Cairn Git objects, repository trees, receipts, or ancestry are reused**. Its base is one empty synthetic root commit, authored as `Synthetic COB fixture <cob-fixture@example.invalid>` with repository-local Git configuration. The bundle contains exactly that root and two signed COB operations (import and ordinary update).

`manifest.ncl` remains the typed source of truth; `manifest.json` is exported by the exact reviewed Nickel evaluator. The ten Cairn evidence receipts are retained byte-for-byte from CLI stdout. The eleventh evidence file, `evidence/transport.json`, is the Nickel-exported projection of typed, measured shell observations in `transport.ncl`, **not a Cairn-emitted transport receipt**. The replay measures its Git refs, branch heads, remotes, tags, file equality, and BLAKE3 digest. The artifact-tree digest is BLAKE3 of the UTF-8 JSON map of relative artifact paths to text, with keys sorted and separators `,` and `:`, encoded as `blake3-` plus standard base64. It is not a claim about a Nix store tree hash.

The scenario previews and executes signed import; changes only `tasks.md`; previews and executes signed update; creates and verifies the exact two-head bundle; imports it into two independently initialized Git stores without remotes or object alternates; observes secondary status; captures a blocked checkout with the native `.cairn/changes/` directory absent; creates that directory; previews then executes checkout; inspects the conflict-free state; and independently reloads signed state from the bundle in the third store. It checks file equality, state/operation bindings, the empty root, and the complete three-commit DAG before retaining output. A preview is checked not to materialize files, and the negative checkout is checked not to mutate the destination.

The retained run produced nine positive Cairn `PASS` receipts and the expected missing-native-directory `FAIL` receipt (`blocked=true`, `mutated=false`), plus valid measured transport observations. Git bundle verification and `fsck --full --no-reflogs` succeeded; the complete reachable graph has three commits and two advertised heads. Both transported stores reduced the same signed object and final state. The CLI was built offline/locked from the immutable reviewed source in the pinned development shell. These observations do **not** claim that the separate Nix package build passed.

## Replay

From the Octet root, supply explicit reviewed executables and source identity:

```sh
CAIRN_COMMIT=6b7e19531554714b6760e8cacc2a980e576d36af
TMPDIR="${TMPDIR:-/var/tmp}" python3 tools/replay-cairn-cob-pilot.py \
  --cairn "$CAIRN" \
  --keys "$OCTET_COB_FIXTURE_KEYS" \
  --policy "$CAIRN_POLICY" \
  --nickel "$NICKEL" \
  --cairn-commit "$CAIRN_COMMIT"
```

`CAIRN_COMMIT` must be the full reviewed source commit used to build `CAIRN`; the script does not infer source provenance from an executable. `NICKEL` must report `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`, the reviewed cohort described in `docs/nickel-toolchain.md`. The signer helper is the repository's `octet-cob-fixture-keys` executable: its single output-directory argument creates `creator` and `delegate` keystores from public fixture seeds 51/52 and prints their public actor IDs. Only the creator signs this ordinary-update scenario. **These public, reproducible fixture keys are never production authority.** No keystores are retained in the fixture, and temporary stores and keys are removed on exit.

The script also requires `git` and `b3sum` on `PATH`. `--output /absolute/path` writes a separate replay instead of replacing this fixture. All Cairn calls run from the temporary scenario root with illustrative relative repository, keystore, root, and policy paths; no signed receipt is edited or redacted. Global/system Git configuration is disabled, and no network transport, remote publication, or API authentication is performed. The replay prints receipt verdicts, measured object/state/bundle identities, no-remotes facts, and BLAKE3 digests of the supplied executables and policy. Fresh signed operation timestamps may change byte identities; regeneration must refresh the typed manifest and projections together, not transplant old hashes.

An unexpected CLI verdict aborts publication of the new fixture and preserves captured receipt bytes under a separately printed `octet-cob-pilot-refusal-*` temporary directory for diagnosis. That debug directory contains no signing keys or Git store and is not successful pilot evidence.

## Validate retained evidence

```sh
git bundle verify fixtures/cairn-cob-pilot/octet-cob.bundle
nix develop -c cargo test --test cairn_cob_pilot
nix develop -c nickel export --format json fixtures/cairn-cob-pilot/manifest.ncl \
  > /tmp/cairn-cob-pilot-manifest.json
cmp fixtures/cairn-cob-pilot/manifest.json /tmp/cairn-cob-pilot-manifest.json
nix develop -c nickel export --format json fixtures/cairn-cob-pilot/transport.ncl \
  > /tmp/cairn-cob-pilot-transport.json
cmp fixtures/cairn-cob-pilot/evidence/transport.json /tmp/cairn-cob-pilot-transport.json
```

This evidence proves local observations for the pinned inputs only. It does not prove Radicle runtime participation, remote publication, authentication, authorization, availability, durability, synchronization, convergence, lifecycle validity, evidence truth, source correctness, whole-program correctness, or release eligibility.
