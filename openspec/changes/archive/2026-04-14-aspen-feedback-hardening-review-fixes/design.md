# Design: Fix Review Gaps In Aspen Feedback Hardening

## Context

`aspen-feedback-hardening` already produced the reusable rollout kit, but its
proof was weaker than the resulting docs. The gaps were operational, not
architectural:

1. the template required `git` and `realpath`, while the doc only named part of
   that environment surface
2. the archive showed syntax/help checks and summary assertions, but not an
   actual phase-1 reconstruction run from the new templates
3. the verification log referenced inline Python snippets without storing the
   full scripts anywhere reviewable

We need to fix the kit and the audit trail together.

## Goals / Non-Goals

**Goals:**

- Make documented assumptions match actual template behavior
- Remove avoidable template dependencies when a documented dependency can do the
  same job
- Capture a real reconstruction transcript that follows the new docs/templates
- Store full helper scripts and verification commands in reviewable artifacts

**Non-Goals:**

- Re-open Aspen rollout policy or re-triage lint families
- Change Tigerstyle lint defaults
- Replace Aspen with a second downstream consumer

## Decisions

### 1. Remove `realpath` dependency instead of documenting more shell surface

**Choice:** Replace `realpath` in `templates/tigerstyle-check.sh` with Python
path resolution.

**Rationale:** `python3` is already a hard dependency for parsing
`rust-toolchain.toml`. Reusing it avoids carrying another external assumption in
the public recipe.

**Alternative:** Keep `realpath` and document it. Rejected because the template
already has a portable path-resolution dependency available.

**Implementation:** Use `python3` with `pathlib.Path(...).resolve()` to compute
`TIGERSTYLE_PATH`.

### 2. Keep `git` explicit in the documented assumptions

**Choice:** Document `git` as a template assumption.

**Rationale:** The template uses `git rev-parse --show-toplevel` to anchor the
repo root. That is intentional and worth stating plainly.

**Alternative:** Remove the `git` dependency and require callers to run from the
workspace root. Rejected because repo-root discovery is useful behavior.

**Implementation:** Update `docs/downstream-rollout.md` to list `git` and note
that the rest of the shell surface is standard Unix userland.

### 3. Prove reconstruction in a disposable Aspen worktree

**Choice:** Capture the end-to-end reconstruction in a temporary Aspen git
worktree instead of mutating `/path/to/git/aspen`.

**Rationale:** The doc promises a maintainer can reconstruct phase 1 from the
new templates. A disposable worktree proves that claim without relying on or
altering Aspen HEAD's tracked script/config files.

**Alternative:** Run against Aspen HEAD in place. Rejected because that would
mix current Aspen policy with the archived phase-1 reconstruction story.

**Implementation:** Create a temp worktree, copy the template script and config,
apply the archived phase-1 scope/config, run the default path and the
`aspen-coordination` spotcheck, then save the full transcript and raw outputs.

### 4. Store helper scripts as first-class evidence files

**Choice:** Save the full Python and shell helpers used for validation under the
change's `evidence/` directory.

**Rationale:** Review should not depend on reconstructing omitted here-docs from
memory. Exact helper sources are cheap to store and easy to audit.

**Alternative:** Keep compact `verification.md` summaries only. Rejected because
that was the failure mode the done review caught.

**Implementation:** Add `.py` and `.sh` evidence helpers, rerun them, and update
`verification.md` to reference the concrete files and commands.

## Risks / Trade-offs

**[Longer evidence artifacts]** Real transcripts are larger than summary notes.
**Mitigation:** Keep a concise summary artifact plus raw run logs.

**[Worktree runtime cost]** Reconstruction takes longer than syntax checks.
**Mitigation:** Run it once in a disposable worktree and keep the transcript for
future review.
