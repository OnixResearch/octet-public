## Phase 1: Implementation

- [x] I1 Rewrite `README.md` into a landing page with project summary, first-class quickstart, naming map, and doc index
- [x] I2 Add consumer docs that cover first run, `[workspace.metadata.tigerstyle]`, `[tigerstyle]` / `dylint.toml`, and how `docs/downstream-rollout.md` fits as the deeper rollout playbook
- [x] I3 Add lint-reference docs that group lints by domain and document default levels plus supported suppression/config paths, using `src/lib.rs`, `src/diagnostics.rs`, and `src/config.rs` as source-of-truth inputs
- [x] I4 Add contributor-maintainer docs that map lint, config, and published-entry-point changes to the required source-of-truth files, docs, templates, and OpenSpec touchpoints

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "New visitor finds the first-class run path from README" with `bash -lc 'rg -n "cargo tigerstyle check|cargo-tigerstyle" README.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/readme-run-path.txt`
- [x] V2 Verify scenario "README routes readers to deeper docs by audience" with `bash -lc 'rg -n "docs/.*(consumer|lint|contribut)|consumer|lint reference|contributor" README.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/readme-doc-map.txt`
- [x] V3 Verify scenario "Consumer finds config and rollout guidance without reading source" with `bash -lc 'rg -n "\[workspace.metadata.tigerstyle\]|\[tigerstyle\]|downstream-rollout" README.md docs/*.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/consumer-docs-config.txt`
- [x] V4 Verify scenario "Reader finds lint defaults and suppression guidance without reading source" with `bash -lc 'rg -n "default|allow\(|deny\(|disabled_lints|lint_levels|tigerstyle::" docs/*.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/lint-reference.txt`
- [x] V5 Verify scenario "Reader distinguishes project name from command and namespace" with `bash -lc 'rg -n "cargo-tigerstyle|cargo tigerstyle check|tigerstyle::|\[tigerstyle\]|\[workspace.metadata.tigerstyle\]|TIGERSTYLE_" README.md docs/*.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/naming-contract.txt`
- [x] V6 Verify scenario "Contributor adding a lint finds doc update touchpoints" with `bash -lc 'rg -n "src/lib.rs|src/diagnostics.rs|README.md|docs/|templates/dylint.toml|openspec/specs" docs/*.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/contributor-lint-crosswalk.txt`
- [x] V7 Verify scenario "Contributor adding a config key or published entry point finds doc update touchpoints" with `bash -lc 'rg -n "src/config.rs|cargo-tigerstyle|workspace.metadata.tigerstyle|README.md|docs/|templates/dylint.toml|openspec/specs" docs/*.md'`; save output at `openspec/changes/refresh-readme-and-docs/evidence/contributor-config-entrypoint-crosswalk.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`bash -lc 'printf "README headings\n"; rg -n "^#{1,2} " README.md; printf "\nDocs headings\n"; find docs -maxdepth 1 -name "*.md" | sort | xargs -r -n1 sh -c "printf \"%s\\n\" \"--- $1\"; rg -n \"^#{1,2} \" \"$1\"" sh'`; input=`README.md docs/*.md`; asserts=`a human reviewer can answer the naming question from the refreshed docs without proposing an immediate repo/package rename`; evidence=`openspec/changes/refresh-readme-and-docs/evidence/human-doc-map.txt`

## Verification Coverage

- `Scenario: New visitor finds the first-class run path from README` -> `V1` -> `openspec/changes/refresh-readme-and-docs/evidence/readme-run-path.txt`
- `Scenario: README routes readers to deeper docs by audience` -> `V2` -> `openspec/changes/refresh-readme-and-docs/evidence/readme-doc-map.txt`
- `Scenario: Consumer finds config and rollout guidance without reading source` -> `V3` -> `openspec/changes/refresh-readme-and-docs/evidence/consumer-docs-config.txt`
- `Scenario: Reader finds lint defaults and suppression guidance without reading source` -> `V4` -> `openspec/changes/refresh-readme-and-docs/evidence/lint-reference.txt`
- `Scenario: Reader distinguishes project name from command and namespace` -> `V5` -> `openspec/changes/refresh-readme-and-docs/evidence/naming-contract.txt`
- `Scenario: Contributor adding a lint finds doc update touchpoints` -> `V6` -> `openspec/changes/refresh-readme-and-docs/evidence/contributor-lint-crosswalk.txt`
- `Scenario: Contributor adding a config key or published entry point finds doc update touchpoints` -> `V7` -> `openspec/changes/refresh-readme-and-docs/evidence/contributor-config-entrypoint-crosswalk.txt`
