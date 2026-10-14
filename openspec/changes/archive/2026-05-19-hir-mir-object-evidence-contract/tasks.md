## Phase 1: Scaffold

- [x] I1 Create `.openspec.yaml`, proposal, delta spec, design, and tasks for the HIR/MIR object evidence supplement contract.

## Phase 2: Verification

- [x] V1 Validate `hir-mir-object-evidence-contract` with `openspec validate hir-mir-object-evidence-contract --strict --json`.
- [x] V2 Validate all OpenSpec specs with `openspec validate --all --strict --json`.
- [x] V3 Run `git diff --check` for scaffold hygiene.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`openspec validate hir-mir-object-evidence-contract --strict --json && openspec validate --all --strict --json && git diff --check`; input=`openspec/changes/archive/2026-05-19-hir-mir-object-evidence-contract/`; asserts=`scaffold is complete, HIR/MIR supplements are optional/toolchain-scoped, and no correctness or semantic-equivalence claim is introduced`; evidence=`terminal validation output`

## Change Status

- [x] Complete

## Verification Coverage

- `Scenario: HIR supplement preserves v1 identity boundary` -> `V1`/`V2` -> terminal validation output
- `Scenario: MIR supplement is version and context scoped` -> `V1`/`V2` -> terminal validation output
- `Scenario: Supplement metadata fails closed before implementation claims` -> `V1`/`V2` -> terminal validation output
