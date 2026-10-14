## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`scripts/replay-aspen.sh`; input=`aspen@909e5e9fa`; asserts=`replay task is self-describing`

## Verification Coverage

- `Scenario: Human checkpoint missing evidence path is blocked` -> `H1` -> `openspec/changes/example/evidence/blocking-missing-human-evidence.txt`
