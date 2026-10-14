# Design: Prove The Rollout Kit On A Second Consumer

## Context

Right now the templates are documented, but every durable proof in this repo is
still Aspen-shaped. The next review question is obvious: can another consumer
follow the same kit without Aspen-specific tribal knowledge? We need one saved
answer.

## Goals / Non-Goals

**Goals:**

- use a real non-Aspen Rust repo as the second consumer
- copy `templates/tigerstyle-check.sh` and `templates/dylint.toml` into a
  disposable sandbox rather than editing the source repo in place
- save one replay helper and transcript that show the template-based path end
  to end

**Non-Goals:**

- landing tigerstyle changes in the second consumer repo
- promoting more lint families than the starter profile needs
- making the second consumer the new canonical starter reference

## Decisions

### 1. Use a disposable sandbox over the real repo

**Choice:** Materialize the second consumer in a temp sandbox, copy the
templates there, edit only the copied files, and run the template-driven check
path from that sandbox.

**Rationale:** We want proof, not repo churn in a different project.

### 2. Save the exact replay helper

**Choice:** Record the second-consumer run through a saved shell helper beside
its transcript.

**Rationale:** Future reviewers should be able to rerun the proof exactly,
without reconstructing the sandbox steps from prose.
