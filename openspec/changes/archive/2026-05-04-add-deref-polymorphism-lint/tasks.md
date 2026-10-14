# add-deref-polymorphism-lint Tasks

Status: complete
Open tasks: none

## Implementation

- [x] I1 [covers=r[deref-polymorphism-lint.warn-public-nominal-target]] Add `deref_polymorphism` structure lint for public local `Deref` impls with nominal targets.
- [x] I2 [covers=r[deref-polymorphism-lint.accepted-targets]] Preserve accepted private wrappers, `str`, slice, non-Deref, and macro-boundary cases.
- [x] I3 [covers=r[deref-polymorphism-lint.inventory]] Register lint inventory, diagnostics, docs, templates, rollout lists, and fixture manifests.

## Verification

- [x] V1 [covers=r[deref-polymorphism-lint.warn-public-nominal-target],r[deref-polymorphism-lint.accepted-targets]] Run package/UI checks and save evidence. command=`nix develop -c bash -c 'tmp=$(mktemp -d); trap "rm -rf $tmp" EXIT; CARGO_TARGET_DIR=$tmp cargo test --lib && CARGO_TARGET_DIR=$tmp cargo test --test ui ui -- --ignored --nocapture'` input=repo worktree asserts=`cargo test --lib` and full ignored UI suite pass with `deref_polymorphism` fixture evidence evidence=`openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/package-ui.txt`
- [x] V2 [covers=r[deref-polymorphism-lint.inventory]] Run OpenSpec/spec/policy/diff validation and save evidence. command=`openspec validate add-deref-polymorphism-lint --strict && openspec validate structure --strict && openspec validate --specs --json && nix run .#tigerstyle-policy-validate && git diff --check` input=repo worktree asserts=change/specs/policy/diff checks pass evidence=`openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/spec-policy-diff.txt`

## Human Review Notes

- [x] H1 [covers=r[deref-polymorphism-lint.inventory]] Confirm advisory rollout remains diagnostic-only and not hard-gated. command=`review docs/templates/lint metadata diff` input=changed lint catalog/config files asserts=`deref_polymorphism` is default-warn, diagnostic-only, and commented/disabled in starter rollout examples evidence=`openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/advisory-rollout.txt`

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Public Deref to nominal target warns | `openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/package-ui.txt` |
| Accepted Deref targets do not warn | `openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/package-ui.txt` |
| Inventory/docs/config stay synchronized | `openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/spec-policy-diff.txt`, `openspec/changes/archive/2026-05-04-add-deref-polymorphism-lint/evidence/advisory-rollout.txt` |
