# Cob Collaboration Conflict Recovery Specification

## Purpose

Defines the `cob-collaboration-conflict-recovery` capability.

## Requirements

### Requirement: Octet SHALL create an authorized same-basis conflict
r[octet.cob_conflict_recovery.divergence] Octet SHALL retain bounded evidence that an embedded creator and delegate can independently sign different lifecycle artifact values from the same imported COB basis.

#### Scenario: Two authorized updates diverge from one basis
r[octet.cob_conflict_recovery.divergence.same_basis]
- GIVEN one signed root operation embedding the creator and delegate as immutable object authority
- WHEN separate explicit stores accept different non-noop `tasks.md` updates signed by those actors before exchanging new refs
- THEN both updates SHALL cite the root operation as their artifact basis and SHALL produce distinct signed operation IDs and content roots.

#### Scenario: Explicit Git transport exposes both heads
r[octet.cob_conflict_recovery.divergence.transport]
- GIVEN the creator and delegate namespace refs target their independent signed updates
- WHEN explicit local Git refspecs populate a convergence store without retaining remotes
- THEN Cairn SHALL report both current tasks heads and SHALL NOT select an implicit winner.

### Requirement: Conflict inspection SHALL keep exact values selector-bound
r[octet.cob_conflict_recovery.inspection] Octet SHALL retain default metadata-only conflict inspection separately from explicit bounded head-value inspection.

#### Scenario: Default inspection exposes metadata only
r[octet.cob_conflict_recovery.inspection.metadata]
- GIVEN the convergence store contains both authorized tasks heads
- WHEN `cairn cob conflicts` runs without a head selector
- THEN the receipt SHALL identify both operation IDs and content hashes while `shown_head` remains absent.

#### Scenario: Explicit selectors reveal one exact head each
r[octet.cob_conflict_recovery.inspection.values]
- GIVEN both reported tasks operation IDs
- WHEN the operator invokes `--show-head tasks=REVISION` once for each revision
- THEN each receipt SHALL reveal only the selected value and SHALL preserve the same unresolved conflict metadata.

### Requirement: Ordinary update SHALL refuse unresolved conflicts
r[octet.cob_conflict_recovery.refusal] Octet SHALL demonstrate that ordinary signed update does not resolve or overwrite conflicting lifecycle heads.

#### Scenario: Update fails closed over conflict
r[octet.cob_conflict_recovery.refusal.update]
- GIVEN both authorized tasks heads remain current
- WHEN the creator previews an ordinary `cairn cob update`
- THEN the receipt SHALL be blocked, SHALL report no mutation, and SHALL leave both namespace ref targets unchanged.

### Requirement: Explicit resolution SHALL supersede the complete basis
r[octet.cob_conflict_recovery.resolution] Octet SHALL preview and execute an explicit authorized resolution whose operation supersedes every current conflict head while preserving historical reachability.

#### Scenario: Preview binds the full conflict basis
r[octet.cob_conflict_recovery.resolution.preview]
- GIVEN explicit selection of the creator's tasks revision
- WHEN the creator previews `cairn cob resolve`
- THEN the plan SHALL select that exact revision, SHALL include both divergent operation IDs in the action basis, and SHALL report no mutation.

#### Scenario: Execution preserves selected and losing revisions
r[octet.cob_conflict_recovery.resolution.execute]
- GIVEN the reviewed preview and unchanged policy, repository, signer, heads, and selection
- WHEN the creator executes the same resolution
- THEN the new signed operation SHALL descend from both divergent operations, SHALL make the selected tasks value current, and SHALL keep the losing delegate revision reachable.

### Requirement: Resolved state SHALL reload and materialize without conflict
r[octet.cob_conflict_recovery.reload] Octet SHALL reload the final signed graph into a third explicit store and materialize the selected lifecycle bytes preview-first.

#### Scenario: Third store observes one resolved state
r[octet.cob_conflict_recovery.reload.status]
- GIVEN a complete Git bundle containing the base branch and both final actor refs
- WHEN a third bare store imports those refs and Cairn inspects the object
- THEN status and conflict inspection SHALL report the resolved state and no current conflict while preserving both divergent operations in history.

#### Scenario: Third checkout materializes selected bytes
r[octet.cob_conflict_recovery.reload.checkout]
- GIVEN a third Octet root with no active copy of the change
- WHEN checkout is previewed and then executed from the reloaded store
- THEN only the explicit selected tasks value SHALL materialize and all other lifecycle artifacts SHALL match the resolved COB state.

### Requirement: Conflict evidence SHALL be typed and fail closed
r[octet.cob_conflict_recovery.evidence] Octet SHALL bind exact actors, refs, operations, bases, hashes, receipts, bundle heads, and non-claims in a typed Nickel manifest validated by deterministic Rust and Nix checks.

#### Scenario: Exact retained evidence validates
r[octet.cob_conflict_recovery.evidence.positive]
- GIVEN the generated manifest, retained receipts, and complete Git bundle
- WHEN focused Rust and Nix validators run
- THEN they SHALL accept exact cross-receipt identities, both conflict heads, the selected head, complete resolution ancestry, reload state, and bounded non-claims.

#### Scenario: Tampered resolution basis is rejected
r[octet.cob_conflict_recovery.evidence.negative]
- GIVEN otherwise valid retained evidence whose losing operation or resolution basis is replaced
- WHEN the deterministic validator runs
- THEN it SHALL reject the evidence with a stable diagnostic and SHALL NOT promote it to merge correctness, synchronization, convergence, durability, or release evidence.

### Requirement: Conflict recovery claims SHALL remain bounded
r[octet.cob_conflict_recovery.boundary] Octet SHALL describe the drill as local signed Git COB integration evidence only.

#### Scenario: Local resolution does not become a distributed claim
r[octet.cob_conflict_recovery.boundary.non_claims]
- GIVEN divergence, inspection, refusal, resolution, reload, and checkout observations pass
- WHEN an operator reviews the fixture
- THEN Octet SHALL NOT claim Radicle runtime participation, publication, remote authorization, availability, durability, synchronization, distributed convergence, semantic merge correctness, source correctness, whole-program correctness, or release eligibility.
