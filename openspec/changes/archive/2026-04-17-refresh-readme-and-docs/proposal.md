## Why

`README.md` is trying to be landing page, lint catalog, migration note, rollout guide, and contributor note at the same time. New consumer entry points now exist (`cargo tigerstyle check` and `nix run ...#cargo-tigerstyle -- check`), but first-time readers still hit Aspen-specific rollout history and long lint tables before they get a simple answer to "what is this" and "how do I run it".

We also only ship one repo-local docs page, `docs/downstream-rollout.md`. That page is useful, but it is Aspen-heavy and assumes the reader already understands the project, the first-class runner, and the naming surface. Aspen rollout work made this gap obvious: the repo now has a first-class consumer engine plus multiple archived rollout references, yet the docs still read like one long implementation log.

The recent rename question is part of the same problem. Current names are load-bearing across the repo and downstream consumers (`tigerstyle`, `cargo-tigerstyle`, `cargo tigerstyle check`, `tigerstyle::...`, `[tigerstyle]`), but we do not explain how those names fit together.

## What Changes

- Restructure `README.md` into a short landing page with project summary, zero-to-first-run path, naming map, and doc index.
- Add audience-specific docs for consumer quickstart/config, lint catalog/reference, and contributor-maintainer workflow.
- Document the current naming contract and explicitly keep a repo/package rename out of scope for this change.
- Add contributor guidance listing which docs, templates, and spec files must change when lints, config keys, or published entry points change.

## Capabilities

### New Capabilities

- `documentation-landing-page`
- `documentation-audience-split`
- `documentation-naming-contract`
- `documentation-maintenance-crosswalk`

## Impact

- **Files**: `README.md`, `docs/*.md`, linked templates, `openspec/specs/documentation/spec.md`
- **APIs**: none
- **Dependencies**: none
- **Testing**: deterministic doc audits plus one human naming review
