# Tasks: Prove The Rollout Kit On A Second Consumer

## Phase 1: Docs and spec

- [x] Extend the documentation spec so the rollout kit requires one saved non-Aspen consumer proof (`specs/documentation/spec.md`)
- [x] Update `docs/downstream-rollout.md` with a pointer to the second-consumer replay helper and transcript

## Phase 2: Proof

- [x] Pick a non-Aspen Rust consumer and create a disposable sandbox that copies `templates/tigerstyle-check.sh` and `templates/dylint.toml`
- [x] Save a replay helper and transcript showing the copied templates run end to end on that second consumer (`evidence/replay-second-consumer.sh`, `evidence/second-consumer-proof.txt`, `evidence/second-consumer-check.txt`)
