# Release operator smoke transcript

This executable transcript is the release-facing smoke index for operator surfaces that should be discoverable from README/help before a release.

```octet-transcript
$ cargo octet check --cache --artifact-dir artifacts/octet --output-format json
=> effect-summary output is written in artifacts/octet/summary.txt
=> lint-cache status is recorded per finding as cache_status=miss|hit|bypassed

$ cargo octet artifact verify --artifact-dir artifacts/octet --output-format json
=> artifact verifier checks saved sidecar hashes, provenance coverage, and manual-review actions without mutating artifacts

$ cargo octet fingerprint --check --review-manifest artifacts/fingerprint-review.json src/lib.rs
=> fingerprint records stable semantic identity and BLAKE3 review hashes

$ cargo octet migrate start --manifest artifacts/octet-migration.json
=> migrate writes a deterministic manifest for safe operator review

$ cargo octet transcript docs/examples/release-operator-smoke/transcript.md
=> executable transcript validation checks command blocks and expected-output assertions

$ cargo octet roadmap artifact store --kind prompt --input prompt.json --store-dir store
=> roadmap surfaces expose content-addressed artifacts, ability checks, pointer rollback, typed sessions, structured todos, and pure-cache receipts

$ nix build .#checks.x86_64-linux.docs-examples --no-link -L
=> release smoke transcript is covered by the docs-example gate
```
