## Context

The standards material and checker foundation now live in `tigerstyle`, not a standalone legacy project. Existing Tiger Style lints remain a Dylint plugin, while standards policy needs a broader traceability and fixture-check surface that should not depend on rustc internals.

## Goals / Non-Goals

**Goals:**
- Rename the active standards codification plan to Tiger Style terminology.
- Keep the Dylint lint engine and standards policy checker as separate seams.
- Preserve source traceability from `docs/standards/source/*.adoc` into Nickel policy entries and diagnostics.
- Prove the migrated foundation with deterministic command outputs.

**Non-Goals:**
- No full implementation of every future rule family in this wave.
- No merge of `crates/tigerstyle-standards` into the Dylint plugin crate.
- No claim that fitness reporting, dependency audit fixtures, or final single-command verification are complete until separate follow-up changes land.

## Decisions

### 1. Keep standards checking in `crates/tigerstyle-standards`

**Choice:** Use a new Rust workspace member for standards policy models, pure rule functions, fixture validation, renderers, and the `tigerstyle-standards` CLI shell.

**Covers:** Codifiable source becomes policy; Blocking hard rule fails deterministically; Advisory rule reports without blocking; Hard rule has fixture evidence; Narrow suppression is accepted; Blanket suppression is rejected; Fixture regression blocks CI.

**Rationale:** These checks operate on policy JSON and source text fixtures. They do not need rustc HIR, so placing them in the Dylint crate would add coupling without precision gains.

**Alternative considered:** Fold policy checks into `src/` with the lints. Rejected because it would mix compiler-plugin code with filesystem/JSON/CLI policy plumbing.

**Implementation:** `crates/tigerstyle-standards/src/runner.rs` exposes `run_checks(CheckInput) -> CheckReport`; `src/main.rs` handles file traversal, JSON parsing, report emission, and exit codes.

**Boundary cases:** Architecture checks currently validate the planned FCIS convention and fixtures using `crates/tigerstyle-core` / `crates/tigerstyle-std` path suffixes. Those suffixes are a standards convention and fixture target, not a claim that the current lint crate already has those live workspace members.

### 2. Author policy registry in Nickel

**Choice:** Keep policy registry contracts in `standards/policies/contracts.ncl` and authored policies in `standards/policies/registry.ncl`.

**Covers:** Missing traceability is rejected; Valid Nickel policy exports for tools; Invalid policy fails before checks run; Candidate rule with weak signal becomes advisory; Human-only guidance stays explicit.

**Rationale:** Nickel gives typed, closed configuration with deterministic JSON export for the Rust checker and shell checks.

**Alternative considered:** Encode all policy metadata directly in Rust. Rejected because documentation, source anchors, checker ownership, and planned evidence need reviewable config diffs.

**Implementation:** Nix apps `tigerstyle-policy-export` and `tigerstyle-policy-validate` wrap Nickel export/typecheck and source-heading validation.

**Boundary cases:** Human-process-only and advisory entries stay in the registry even when no blocking checker exists, so review can see non-automated standards without build noise.

### 3. Treat Dylint as one checker owner among several

**Choice:** Policies can name `checker_id = "tigerstyle"` when an existing lint owns enforcement, while source-shape rules name `checker_id = "tigerstyle-standards"`.

**Covers:** Hard rule has fixture evidence; Local-first standards verification succeeds; Fixture regression blocks CI.

**Rationale:** The combined project should expose one Tiger Style product, but still preserve internal seams by checker ownership.

**Alternative considered:** Rename all checker ownership to `tigerstyle-standards`. Rejected because Dylint lint diagnostics and standards source-shape diagnostics are different enforcement layers.

**Implementation:** `standards/policies/contracts.ncl` declares both checker IDs; README and contributor docs describe both command surfaces.

**Boundary cases:** Policies backed by future checker owners remain planned/deferred until they get fixtures and deterministic verification.

### 4. Rename source traceability and suppressions

**Choice:** Migrate source anchors to `docs/standards/source/*.adoc`, policy IDs to `tigerstyle.*`, and suppressions to `tigerstyle-allow:`.

**Covers:** Legacy project names are absent from Tiger Style standards artifacts; Narrow suppression is accepted; Blanket suppression is rejected.

**Rationale:** The user explicitly asked for legacy project naming to be removed. Keeping old names in OpenSpec would make implementation and spec incoherent.

**Alternative considered:** Keep old source names as historical aliases. Rejected because it would preserve the wrong project boundary in new Tiger Style artifacts.

**Implementation:** Migrated docs, policy fixtures, Rust constants, README entries, and OpenSpec files use Tiger Style names only.

**Boundary cases:** Historical archived OpenSpec changes in this repo are not part of the migrated standards surface and are not renamed by this change.

## Deterministic Proof Plan

- scenario or requirement:
  - Legacy project names are absent from Tiger Style standards artifacts
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/no-legacy-names.txt`
- command / helper:
  - `rg -n -i 'e[v]ermeta' . --glob '*.{rs,md,toml,ncl,adoc,nix,yml,yaml,txt}' || true`
- exact expected output / acceptance signal:
  - no matches outside historical archived OpenSpec paths; migrated standards surface has zero matches
- ordering / prerequisites:
  - all migrated files are staged or present in the worktree
- cleanup / postcondition:
  - evidence file remains checked in with the change

- scenario or requirement:
  - Valid Nickel policy exports for tools; Invalid policy fails before checks run; Missing traceability is rejected
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`
- command / helper:
  - `nix run .#tigerstyle-policy-validate`
- exact expected output / acceptance signal:
  - command exits 0
- ordering / prerequisites:
  - `standards/policies/registry.ncl` and fixtures exist
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - Blocking hard rule fails deterministically; Advisory rule reports without blocking; Hard rule has fixture evidence; Narrow suppression is accepted; Blanket suppression is rejected
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- command / helper:
  - `nix develop -c bash -c 'export CARGO_TARGET_DIR=target/verify-standards; cargo test -p tigerstyle-standards'`
- exact expected output / acceptance signal:
  - 19 unit tests pass and doc-tests pass
- ordering / prerequisites:
  - crate is a workspace member
- cleanup / postcondition:
  - target artifacts stay under repo-local `target/verify-standards`

- scenario or requirement:
  - Fixture regression blocks CI
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/flake-standards-checks.txt`
- command / helper:
  - `nix build .#checks.x86_64-linux.policy-registry --no-link && nix build .#checks.x86_64-linux.standards-fixtures --no-link`
- exact expected output / acceptance signal:
  - both builds exit 0
  - `standards-fixtures` asserts at least one `tigerstyle.architecture.external-domain-leakage` diagnostic from the architecture noncompliant fixture set
  - `standards-fixtures` asserts the thick-root fixture emits `tigerstyle.architecture.root-crate-thinness` and does not also emit `tigerstyle.architecture.public-api-budget`
- ordering / prerequisites:
  - new files are staged so Git-backed flake source includes them
- cleanup / postcondition:
  - no `result` link is created because `--no-link` is used

- scenario or requirement:
  - Local-first standards verification succeeds
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/verification-rails.txt`
- command / helper:
  - `nix build .#checks.x86_64-linux.fmt --no-link && nix build .#checks.x86_64-linux.clippy --no-link && nix build .#checks.x86_64-linux.nextest --no-link && nix build .#checks.x86_64-linux.ui --no-link`
- exact expected output / acceptance signal:
  - all four builds exit 0
  - the rails cover formatting, Clippy denial, workspace tests, and Tiger Style lint UI checks as separate flake seams
- ordering / prerequisites:
  - changed files are staged so Git-backed flake source includes them
- cleanup / postcondition:
  - no `result` link is created because `--no-link` is used

- scenario or requirement:
  - Candidate rule with weak signal becomes advisory
- artifact path:
  - `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/advisory-classification.txt`
- command / helper:
  - `nix run .#tigerstyle-policy-export > "$registry_json"` followed by `jq` field extraction and `nix run .#tigerstyle-standards -- check --registry "$registry_json" --source standards/check-fixtures/architecture/advisory --family architecture --output json`
- exact expected output / acceptance signal:
  - `policy_id=tigerstyle.architecture.public-api-budget`
  - `classification=advisory`
  - `enforcement_level=advisory`
  - `default_severity=warning`
  - `check_exit=0`
  - `diagnostics=1`
  - `severity=warning`
  - `blocking=false`
- ordering / prerequisites:
  - registry export and advisory fixture are present
- cleanup / postcondition:
  - temp registry/report files are removed

## Risks / Trade-offs

- Hard-coded FCIS suffixes could be misread as current workspace layout -> docs and design state they are fixture/convention targets for future split checks.
- Broad future policy inventory can outpace implemented checks -> tasks keep future rule families open instead of marking the whole change complete.
- Historical legacy project artifacts may remain in the old repo -> Tiger Style migration removes the naming from new Tiger Style artifacts and deletes the old active OpenSpec change from the source repo.
