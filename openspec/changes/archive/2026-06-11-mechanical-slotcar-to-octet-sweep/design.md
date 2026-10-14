## Context

The archived `2026-05-18-rename-slotcar-to-octet` change established Octet as canonical, but active files still contain `slotcar` in maintained specs, command/config compatibility paths, standards crate names, fixtures, Nix outputs, and docs. Existing `tools/classify-slotcar-rename.py` classifies names after the fact; this change needs a Steel-first mechanical path that can plan, apply, and verify the sweep without rewriting historical evidence.

## Goals / Non-Goals

**Goals:**
- Use Steel as the deterministic scan/rewrite/report engine for active `slotcar` -> `octet` identity replacement.
- End with zero unapproved active `slotcar` / `Slotcar` product-identity occurrences.
- Make every retained compatibility occurrence explicit, justified, and tested.
- Keep archived evidence historically faithful.

**Non-Goals:**
- Changing lint behavior or policy semantics.
- Rewriting archived transcripts and historical proof bundles.
- Replacing legitimate `Tiger Style` philosophy references.

## Decisions

### 1. Build a Steel helper with a pure rewrite core and thin shell

**Choice:** Add `tools/octet-identity-sweep.scm` with pure functions for file eligibility, token classification, replacement planning, report rendering, and zero-blocked validation. The shell only reads files, writes apply changes, and emits reports.

**Covers:** `Steel dry-run inventories active slotcar references`; `Steel apply rewrites active documentation identity`; `Remaining slotcar terms are explicitly allowed`; `Steel sweep handles command and package surfaces`.

**Obligation cross-check:** The proposal requires dry-run/apply/verify reports, exact path/line reporting, no dry-run mutation, and zero unapproved active rows. The helper must expose modes for each obligation and keep report sorting deterministic by repo-relative path, line, column, and matched text.

**Rationale:** Steel keeps the mechanical edit plan reviewable and repeatable inside Pi without ad-hoc shell pipelines.

**Alternative considered:** Keep extending `tools/classify-slotcar-rename.py`. Rejected because the requested workflow is Steel-based and the existing helper is classification-only.

**Implementation:** Represent each occurrence as a record with path, line, column, match, replacement, category, and reason. Apply mode rewrites only records categorized as `rewritten`.

**Boundary cases:** Binary/large generated files are skipped with an explicit skip category; dry-run must leave the working tree unchanged; path renames that cannot be expressed as text edits are emitted as required follow-up actions.

### 2. Separate active replacement from archive classification

**Choice:** Active non-archived files are candidates for replacement. Archived evidence under `openspec/changes/archive/**`, `cairn/archive/**`, and similar historical bundles is scanned but not rewritten.

**Covers:** `Archived evidence remains historical`; `Steel apply rewrites active documentation identity`; `Remaining slotcar terms are explicitly allowed`.

**Obligation cross-check:** The proposal excludes archive mutation while requiring a final report. The report must still count archive occurrences as `allowed-archive` so reviewers can distinguish them from missed active files.

**Rationale:** Archive rewrites would invalidate historical command transcripts and saved proof paths.

**Alternative considered:** Rewrite every archived `slotcar` string too. Rejected because it corrupts evidence provenance and produces unreviewable churn.

**Implementation:** Add path-scope predicates before replacement rules. Current specs and docs are active; archived change/evidence directories are classify-only.

**Boundary cases:** If an active spec points at a dead active change path, update that current spec reference to an archived path or Octet path; do not modify the archived artifact itself.

### 3. Retire by default, allow compatibility only by explicit rule

**Choice:** The sweep treats active `slotcar` command, config, namespace, package, receipt, and suppression-marker spellings as blocked unless a rule marks them `allowed-compatibility` with rationale and a focused test.

**Covers:** `Compatibility retirement or allowlist is explicit`; `Octet config and command paths stay functional`; `Steel sweep handles command and package surfaces`.

**Obligation cross-check:** The proposal asks to retire aliases where mechanical updates can cover callers and to justify retained legacy strings. This decision prevents silent compatibility drift.

**Rationale:** Remaining `slotcar` names are public API. They need an explicit keep/remove decision, not accidental survival.

**Alternative considered:** Keep every current alias. Rejected because it fails the user's all-reference sweep goal and leaves stale current specs.

**Implementation:** Update docs/specs/templates/tests for Octet spellings; rename standards package/path surfaces when required; keep any intentionally retained legacy parser or receipt normalizer behind a named allow category plus a test.

**Boundary cases:** Removing aliases is a breaking surface change; the final human/oracle checkpoint reviews the retained/retired table before the change is marked complete.

### 4. Make proof artifacts self-checking

**Choice:** Save the Steel dry-run report, apply receipt, final verify report, compatibility decision table, and focused Cargo/Nix/doc check transcripts under the change evidence directory.

**Covers:** `Steel dry-run inventories active slotcar references`; `Remaining slotcar terms are explicitly allowed`; `Octet config and command paths stay functional`; `Compatibility retirement or allowlist is explicit`.

**Obligation cross-check:** Every scenario needs a deterministic artifact with an exact acceptance signal. Reports must include `blocked: 0` for final verification and exact rewrite counts for apply.

**Rationale:** The replacement is broad enough that reviewers need machine-readable proof, not prose claims.

**Alternative considered:** Use only `grep -R slotcar`. Rejected because it cannot distinguish active leaks from archives or compatibility.

**Implementation:** The Steel helper renders text and JSON reports; V-tasks capture text artifacts and tests assert JSON schema fields.

**Boundary cases:** If Nix checks are blocked by environment constraints, capture the blocker and run the smallest package-level Cargo check that covers the renamed surface.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Steel dry-run inventories active slotcar references`
- artifact path:
  - `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-dry-run.txt`
- command / helper:
  - `pi steel file=tools/octet-identity-sweep.scm args='["dry-run","--format","text"]'`
- exact expected output / acceptance signal:
  - report includes `mode: dry-run`, at least one active occurrence before apply, and `files_modified: 0`
- ordering / prerequisites:
  - helper implemented before any apply run
- cleanup / postcondition:
  - working tree content is unchanged by dry-run

- scenario or requirement:
  - `Scenario: Steel apply rewrites active documentation identity`
  - `Scenario: Steel sweep handles command and package surfaces`
- artifact path:
  - `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-apply.txt`
- command / helper:
  - `pi steel file=tools/octet-identity-sweep.scm args='["apply","--format","text"]'`
- exact expected output / acceptance signal:
  - report includes `mode: apply`, positive rewritten file and occurrence counts, and no archive rewrites
- ordering / prerequisites:
  - dry-run reviewed first
- cleanup / postcondition:
  - generated fixtures and lockfiles are refreshed after path/package renames

- scenario or requirement:
  - `Scenario: Remaining slotcar terms are explicitly allowed`
  - `Scenario: Archived evidence remains historical`
- artifact path:
  - `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-final-verify.txt`
- command / helper:
  - `pi steel file=tools/octet-identity-sweep.scm args='["verify","--format","text"]'`
- exact expected output / acceptance signal:
  - report includes `blocked: 0`, `unapproved_active_slotcar: 0`, and archive rows only under `allowed-archive`
- ordering / prerequisites:
  - apply mode and generated-artifact refresh complete
- cleanup / postcondition:
  - no unapproved active `slotcar` strings remain

- scenario or requirement:
  - `Scenario: Compatibility retirement or allowlist is explicit`
- artifact path:
  - `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/compatibility-decisions.txt`
- command / helper:
  - `pi steel file=tools/octet-identity-sweep.scm args='["compatibility-report","--format","text"]'`
- exact expected output / acceptance signal:
  - each retained compatibility row has a rationale and a named focused test; retired rows name the updated replacement surface
- ordering / prerequisites:
  - final verify report exists
- cleanup / postcondition:
  - human/oracle checkpoint can review one bounded table

- scenario or requirement:
  - `Scenario: Octet config and command paths stay functional`
- artifact path:
  - `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/focused-checks.txt`
- command / helper:
  - `nix develop -c bash -c 'cargo test -p cargo-octet octet_identity_surfaces -- --nocapture && cargo test -p octet-standards octet_identity_surfaces -- --nocapture && git diff --check'`
- exact expected output / acceptance signal:
  - focused tests pass and `git diff --check` exits successfully
- ordering / prerequisites:
  - renamed packages/tests compile after apply
- cleanup / postcondition:
  - any blocked Nix/package issue is recorded with the next smallest passing check

## Risks / Trade-offs

- Broad rename churn -> keep the Steel dry-run report reviewable before apply and split compatibility decisions into a separate table.
- Compatibility breakage -> require explicit retained/retired rows plus focused tests.
- Archive corruption -> make archive paths classify-only and assert no archive rewrites in the apply receipt.
- Generated artifact drift -> refresh fixtures/lockfiles after path renames and include them in focused checks.
