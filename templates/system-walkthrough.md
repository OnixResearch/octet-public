# System walkthrough template

Fill every field. Keep answers concrete.

## Intended reader and system slice

- Artifact under review:
- Owner / reviewer:
- Intended reader:
- Boundary or subsystem under review:
- System slice in scope:
- Related slice or dependency out of scope:
- Why this slice matters now:

## Major components and boundaries

- Component 1 and role:
- Component 2 and role:
- Component 3 and role:
- Ownership boundary between components:
- Interface or contract that matters most:
- Boundary that is easiest to misunderstand:

## Key relationships and dependency edges

- Upstream dependency or caller:
- Downstream dependency or callee:
- Shared state, queue, or storage edge:
- External system or crate boundary:
- Dependency edge that drives most risk:
- Assumption about that edge that still needs review:

## Primary end-to-end flow

- Entry point:
- Trigger or input:
- Step 1:
- Step 2:
- Step 3:
- Exit condition or output:
- Failure or degradation branch that reviewer should inspect next:

## Deep dives and weak spots

- Deep dive area 1:
- Deep dive area 2:
- Weak spot or confusing boundary:
- Ownership gap or missing document:
- Assumption most likely to break under change or load:
- Extra evidence, code path, or diagram still needed:

## Open questions and follow-ups

- Open question 1:
- Open question 2:
- Follow-up change or document:
- Owner for follow-up:
- Review verdict:
- Next action:
