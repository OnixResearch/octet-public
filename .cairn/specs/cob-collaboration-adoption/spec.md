# Cob Collaboration Adoption Specification

## Purpose

Defines the `cob-collaboration-adoption` capability.

## Requirements

### Requirement: Octet SHALL exercise preview-first signed COB mutation
r[octet.cob_collaboration.pilot] Octet SHALL retain bounded downstream evidence that a native change can be imported and updated through Cairn's `radicle-cob.v1` backend with explicit repositories and mutation keys.

#### Scenario: Native change import is previewed before execution
r[octet.cob_collaboration.pilot.import]
- GIVEN a valid active Octet Cairn change, a bare Git store seeded from the declared base revision, and an explicit local mutation keystore
- WHEN the operator previews and then executes `cairn cob import`
- THEN the preview SHALL declare the planned signed mutation and the executed receipt SHALL bind the resulting object, operation, actor, ref, content root, and non-claims.

#### Scenario: Changed lifecycle artifacts produce a signed update
r[octet.cob_collaboration.pilot.update]
- GIVEN the imported object and an authorized embedded creator whose local lifecycle artifacts changed after import
- WHEN the operator previews and then executes `cairn cob update`
- THEN the update SHALL be non-noop, SHALL append an authorized operation, and SHALL retain the original signed history.

### Requirement: Octet SHALL keep transport outside Cairn
r[octet.cob_collaboration.transport] Octet SHALL move COB state only through explicit Git ref transport in this pilot and SHALL NOT invoke Radicle node, profile, discovery, seeding, announcement, connection, or synchronization behavior.

#### Scenario: Explicit Git transport preserves exact refs
- GIVEN a source store containing the base branch and signed namespaced COB refs and an empty destination store
- WHEN Git fetches the explicit branch and namespace refspecs by local path
- THEN source and destination ref targets SHALL match exactly and neither COB store SHALL retain a configured remote.

### Requirement: A second Octet checkout SHALL materialize transported state
r[octet.cob_collaboration.checkout] Octet SHALL demonstrate status and preview-first checkout against the transported store from a second repository root.

#### Scenario: Transported change materializes without implicit winner selection
- GIVEN a second Octet root at the declared base revision with no local active change and a transported conflict-free COB object
- WHEN the operator inspects status, previews checkout, and executes checkout
- THEN the materialized active-change artifacts SHALL byte-match the source artifacts and conflict inspection SHALL report no conflicting heads.

### Requirement: Pilot evidence SHALL be typed and fail closed
r[octet.cob_collaboration.evidence] Octet SHALL bind the pilot's exact identities and receipt hashes in a typed Nickel manifest and SHALL validate cross-receipt consistency through a deterministic pure core.

#### Scenario: Current evidence validates
- GIVEN the generated manifest and all retained signed mutation, status, checkout, conflict, and transport receipts
- WHEN the focused evidence test runs
- THEN it SHALL accept exact matching command, backend, base, actor, object, ref, operation, content-root, transport, and artifact identities.

#### Scenario: Tampered evidence is rejected
- GIVEN otherwise valid evidence with a changed object, ref target, actor, content root, receipt hash, or non-claim boundary
- WHEN the same deterministic validator runs
- THEN it SHALL reject the evidence with a stable diagnostic and SHALL NOT promote the pilot to publication, synchronization, convergence, durability, correctness, or release evidence.

### Requirement: Validation SHALL preserve dependency and claim boundaries
r[octet.cob_collaboration.validation] Octet SHALL validate the pilot without adding Cairn, `radicle`, or `radicle-node` to its product dependency graph.

#### Scenario: Bounded release rails run
- GIVEN the completed pilot artifacts and the repository's pre-existing validation baseline
- WHEN focused tests, lifecycle gates, dependency checks, Octet checks, and selected Nix gates run
- THEN the pilot's added checks SHALL pass and unrelated baseline failures SHALL remain explicitly reported rather than silently repaired or attributed to the pilot.

### Requirement: Pilot receipts SHALL remain non-authoritative
r[octet.cob_collaboration.boundary] Octet SHALL describe this evidence as bounded local integration observations only.

#### Scenario: Local success does not become a distributed claim
- GIVEN signed import, update, Git transport, status, checkout, and conflict-free observations pass
- WHEN an operator reviews the pilot
- THEN Octet SHALL NOT claim Radicle runtime participation, remote publication, peer availability, synchronization, convergence, durable storage, identity-document authority, source correctness, whole-program correctness, or release eligibility.
