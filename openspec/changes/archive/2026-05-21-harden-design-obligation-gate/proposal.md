## Why

Repo-local review metrics still show design-stage omission findings as the top repeated class (`omission|design|prompt`, 80 hits). The existing design template asks decisions to list covered scenarios, but repeated findings show a narrower failure: proposal promises and delta-spec obligations can still disappear from `design.md` unless the design explicitly cross-checks them.

## What Changes

- Strengthen the design template with an explicit obligation cross-check prompt.
- Strengthen `openspec/config.yaml` design-stage guidance to require proposal/spec obligation mapping, not only scenario coverage.
- Add design-governance fixtures that preserve one missing-obligation blocked case and one covered pass case for future gate review.

## Capabilities

### Modified Capabilities
- `openspec-stage-authoring`: design-stage authoring now requires visible cross-checking of proposal verification expectations, delta-spec scenarios, boundary/negative-control obligations, and proof artifacts.

## Impact

- **Files**: `openspec/templates/design.md`, `openspec/config.yaml`, `openspec/specs/openspec-stage-authoring/spec.md`, `tests/fixtures/openspec-design-governance/**`.
- **APIs**: no Rust API changes.
- **Dependencies**: none.
- **Testing**: deterministic fixture-shape helper output, focused design fixture gate transcript, OpenSpec validation, and `git diff --check`.
