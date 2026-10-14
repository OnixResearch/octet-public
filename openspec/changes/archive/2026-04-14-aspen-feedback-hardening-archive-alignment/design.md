# Design: Align Aspen Reconstruction Docs And Archived Evidence

## Context

The current archive already contains a successful Aspen phase-1 reconstruction
transcript. Problem: the public guide still describes a simpler path than the
one the transcript actually needs.

Aspen's workspace depends on sibling repos (`aspen-dns`, `aspen-wasm-plugin`)
that themselves point back at `../aspen`. A naive temporary worktree breaks
that layout. The working proof fixed this with a sandbox parent containing:

- `aspen/` as a worktree at commit `909e5e9fa`
- sibling symlinks `aspen-dns -> /path/to/git/aspen-dns`
- sibling symlinks `aspen-wasm-plugin -> /path/to/git/aspen-wasm-plugin`

The docs need to say that plainly, and the archived transcripts need to point at
archive paths instead of the deleted active change.

## Goals / Non-Goals

**Goals:**

- make the published Aspen reconstruction path match the working proof exactly
- refresh archived transcript command lines so they reference archive paths
- add a simple artifact that shows the final task checklist and its evidence map

**Non-Goals:**

- changing tigerstyle lint behavior
- changing Aspen policy or warning counts
- adding new downstream consumers

## Decisions

### 1. Document the sandbox parent explicitly

**Choice:** Update `docs/downstream-rollout.md` to show the sandbox parent plus
sibling symlink setup before the Aspen worktree is created.

**Rationale:** That is the only path we actually proved. Docs must not imply a
plain worktree is enough when sibling path dependencies make it fail.

**Alternative:** Try to make the plain worktree path succeed. Rejected because
Aspen's sibling-repo layout is real and the sandbox setup is straightforward to
document.

**Implementation:** Replace the plain `git worktree add` step with a short shell
sequence that creates the sandbox, symlinks the sibling repos, and checks out
Aspen as `sandbox/aspen`.

### 2. Re-run text artifacts from the archived location

**Choice:** Refresh all archive-facing `.txt` outputs after the change is
archived.

**Rationale:** The current helper scripts and transcripts exist, but some saved
commands still mention the old active-change path. Re-running from the archive
makes the evidence self-describing.

**Alternative:** Hand-edit the old `.txt` files. Rejected because transcripts
should come from real reruns, not text surgery.

**Implementation:** After archiving, run the archive-local helper scripts again
and overwrite the `.txt` outputs in the archive.

### 3. Add an explicit task checklist artifact

**Choice:** Save the archived `tasks.md` plus a short evidence map as a review
artifact.

**Rationale:** Review should not require jumping between the task file and the
verification narrative to understand which checkbox was proven by which
artifact.

**Alternative:** Rely on `verification.md` alone. Rejected because the done
review already reported that this was too indirect.

**Implementation:** Add `evidence/task-checklist.txt` with the task list and the
matching evidence references.

## Risks / Trade-offs

**[More process text]** The reconstruction section gets longer.
**Mitigation:** Keep the Aspen-specific sandbox setup scoped to that one section.

**[Duplicate archive reruns]** Re-running archived text artifacts repeats work.
**Mitigation:** Only refresh the evidence that reviewers actually inspect.
