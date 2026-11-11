# Octet Cairn COB conflict-recovery fixture

This fixture records one bounded downstream conflict drill using Cairn commit `70db8fe114910feb5d162fea5db8ad4ffdb6f176` against Octet base commit `22722030cd734900c565840997a62b36055aa85d`.

`manifest.ncl` is the typed source of truth and `manifest.json` is its generated runtime projection. The Git bundle retains the base branch, the creator's resolved namespace ref, and the delegate's losing namespace ref. `heads/` keeps both exact conflicting `tasks.md` values. `evidence/` retains signed import and same-basis update receipts, metadata-only and selector-bound conflict inspections, ordinary-update refusal, preview-first resolution, third-store reload, checkout, and explicit transport observations.

The creator and delegate keys were generated under ignored `target/` and are not retained. Their public actor IDs are deterministic fixture evidence only and have no production authority. Every bare store used by the drill had repository-local display metadata and no configured remote.

Validate the fixture:

```sh
git bundle verify fixtures/cairn-cob-conflict-pilot/octet-cob-conflict.bundle
nix develop -c cargo test --test cairn_cob_conflict_pilot
nix develop -c nickel export --format json \
  fixtures/cairn-cob-conflict-pilot/manifest.ncl \
  > target/cairn-cob-conflict-pilot-manifest.json
cmp fixtures/cairn-cob-conflict-pilot/manifest.json \
  target/cairn-cob-conflict-pilot-manifest.json
nix build .#checks.x86_64-linux.cairn-cob-conflict-pilot --no-link -L
```

The resolved lifecycle observation is intentionally `stale`: selecting a content head resolves the artifact conflict but does not silently rewrite the earlier lifecycle subject root. This is local signed Git COB integration evidence only. It does not prove semantic merge correctness, Radicle runtime participation, publication, remote authorization, availability, durability, synchronization, distributed convergence, source correctness, whole-program correctness, or release eligibility.
