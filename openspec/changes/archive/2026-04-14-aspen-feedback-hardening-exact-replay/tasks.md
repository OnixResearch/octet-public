# Tasks: Prove The Exact Aspen Replay Path We Document

## Phase 1: Docs and checker alignment

- [x] Update `docs/downstream-rollout.md` so the exact Aspen phase-1 replay path is the archived helper command, with the sandbox/symlink layout described as what the helper performs
- [x] Tighten the saved count checker so it asserts the `aspen-hlc: 0 starter findings` claim explicitly
- [x] Update consistency checks so they verify the final helper-driven replay path and the stricter count checker behavior

## Phase 2: Exact replay evidence

- [x] Re-run the archived helper-driven replay and save a transcript whose top-level command matches the final documented replay path exactly
- [x] Refresh archived verification notes and task-checklist evidence so the checked Phase 2 task is provably backed by the new transcript
