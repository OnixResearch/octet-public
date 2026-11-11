# Octet Cairn COB pilot fixture

This fixture records one bounded downstream use of Cairn commit `70db8fe114910feb5d162fea5db8ad4ffdb6f176` against Octet base commit `28ffbe8cfb01879a241cad248008b75252a8b7f5`.

`manifest.ncl` is the typed source of truth. `manifest.json` is its generated runtime projection. `octet-cob.bundle` retains the exact base branch and signed namespace history for independent Git verification. `evidence/` contains the retained preview and execution receipts for signed import, signed update, explicit local Git ref transport, second-root status, checkout, and conflict inspection. The negative checkout receipt proves that an empty Git clone must recreate the untracked `cairn/changes/` directory before materialization.

The mutation key was generated under ignored `target/` and is not retained. Only public actor and signed object identities are recorded. The primary and destination COB stores were also temporary and had no configured remotes.

Validate the fixture:

```sh
git bundle verify fixtures/cairn-cob-pilot/octet-cob.bundle
nix develop -c cargo test --test cairn_cob_pilot
nix develop -c nickel export --format json fixtures/cairn-cob-pilot/manifest.ncl \
  > target/cairn-cob-pilot-manifest.json
cmp fixtures/cairn-cob-pilot/manifest.json target/cairn-cob-pilot-manifest.json
```

This evidence proves local observations for the pinned inputs only. It does not prove Radicle runtime participation, remote publication, authentication, authorization, availability, durability, synchronization, convergence, lifecycle validity, evidence truth, source correctness, whole-program correctness, or release eligibility.
