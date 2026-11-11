## Context

Tigerstyle currently has Dylint diagnostics, policy registry concepts, hard-gate profiles, function fingerprints, and standards guidance for core/shell, capability vocabulary, content-addressed evidence, and verified logic. The next enforcement step is to make the policy registry capable of saying when a rule is merely observed versus when Rust construction must make the bad shape impossible.

Relevant sibling references are useful but must not become implicit trust shortcuts:

- `../vericoder-mvp/` has a proof-relevant core / std shell split and seed-corpus patterns.
- `../verified-logic/` has Verus-backed primitives and UCAN-related verified modules.
- `../ucan/` has a `#![no_std]` `ucan-core`, a std shell, and a vendored verified-logic bridge.

## Goals / Non-Goals

**Goals:**
- Add policy vocabulary for lint, gate, structural, verified, and human/oracle enforcement.
- Define Nickel-typed policy, mechanism, evidence, suppression, proof-rail, and seed-manifest contracts with cross-field validation.
- Specify how Tigerstyle validates structural Rust mechanisms and proof rails.
- Specify safe vendoring of reference seeds from sibling repositories.
- Keep implementation staged behind fixtures, manifests, and local verification.

**Non-Goals:**
- Do not vendor sibling repo code in this spec-only change.
- Do not claim sibling proofs satisfy Tigerstyle proof obligations without local evidence.
- Do not require every policy to become structural or verified.

## Decisions

### 1. Enforcement mode is policy metadata, not an ad-hoc CLI flag

**Choice:** Add current and target enforcement mode to policy entries, with accepted values `lint`, `gate`, `structural`, `verified`, and `human-oracle`.

**Covers:** Scenario: Lint-only policy remains explicit; Scenario: Target mode stronger than current mode records escalation; Scenario: Machine output classifies enforcement failure type

**Rationale:** Policy-level metadata lets the same checker output, CI gates, docs, and suppression rules agree on what kind of evidence is required.

**Alternative considered:** Add one-off flags such as `--require-no-std-core`. That would work for a single purity slice but would not scale to capability, bounded primitive, type-state, or proof rails.

**Implementation:** Extend policy schema/export first, then teach checkers to consume exported mode/mechanism fields and emit mode-aware findings.

**Boundary cases:** Policies with weak static signal must stay `lint` or `human-oracle`; the design must not promote them just because structural vocabulary exists.

### 2. Nickel is the typed policy authoring surface

**Choice:** Put policy identity, enforcement modes, mechanism-specific obligations, evidence contracts, suppression schema, proof-rail metadata, and seed manifests in closed Nickel contracts. Export deterministic JSON/TOML for Rust tools to consume.

**Covers:** Scenario: Closed Nickel policy contract rejects unknown fields; Scenario: Structural mode requires structural mechanism contract; Scenario: Verified mode requires proof rail contract; Scenario: Deny severity requires deterministic checker and evidence; Scenario: Crate boundary contract exports core and shell obligations; Scenario: Capability contract exports constructor authority; Scenario: Bounded and type-state contracts export API obligations; Scenario: Seed corpus contract exports provenance and role

**Rationale:** Nickel gives typed configuration, defaults, merge semantics, contracts, and exportable machine data. That keeps policy intent reviewable and prevents Rust checkers from reconstructing enforcement intent from scattered prose or ad-hoc flags.

**Alternative considered:** Encode the policy schema directly in Rust or TOML. Rust-only schemas are harder to review as policy, while TOML lacks contract-level cross-field validation such as `structural` requiring a structural mechanism or `verified` requiring proof rails.

**Implementation:** Add `standards/policies/schema.ncl`, enforcement-mode enums, mechanism contracts, policy bundles, and export commands before adding checker behavior. Rust checkers consume the exported machine data and report policy IDs plus mechanism-specific failure kinds.

**Boundary cases:** Nickel remains the policy/config layer, not the source analyzer. Source parsing, crate graph inspection, hash checking, proof execution, and machine-output generation stay in Rust.

### 3. Structural validation proves the claimed Rust mechanism exists

**Choice:** Structural enforcement requires concrete checks for crate graph, crate attributes, forbidden imports, approved capability constructors, bounded primitives, and type-state API surfaces.

**Covers:** Scenario: No-std core boundary is enforced; Scenario: Shell-to-core dependency direction is enforced; Scenario: Capability constructors are shell-owned; Scenario: Type-state protocol surfaces reject raw operations; Scenario: Bounded primitive requirement rejects unbounded replacement; Scenario: CI fails on missing structural evidence

**Rationale:** A policy cannot honestly claim construction-level enforcement unless the checker validates the construction. This turns Tigerstyle from only "find bad calls" into "reject false claims of impossible-by-construction design."

**Alternative considered:** Keep all such checks as ordinary source lints. That misses crate graph and API-shape claims and makes it too easy for config comments to drift from reality.

**Implementation:** Start with a purity/core-shell vertical slice, then add capability, bounded primitive, and type-state fixtures. Each mechanism needs positive and negative fixtures before becoming a hard gate.

**Boundary cases:** Stable crates may require compatibility profiles instead of `#![no_std]`. Compatibility profiles must be explicit and fixture-backed rather than silently weakening structural claims.

### 4. Verified enforcement requires local proof artifacts

**Choice:** Verified mode requires an explicit backend, proof command, source/proof mapping, expected pass signal, and trust-boundary accounting.

**Covers:** Scenario: Verified policy requires proof command; Scenario: Proof-relevant code cannot be imported as trusted by default; Scenario: Verified-logic seed requires local proof boundary

**Rationale:** Verified-logic primitives are valuable seeds, but Tigerstyle must know which proof command and trust boundaries apply in its own repository.

**Alternative considered:** Trust sibling repo green status. That creates stale, non-local proof claims and weakens review.

**Implementation:** Add manifest fields for verified seeds and require saved local transcripts before marking proof-relevant scenarios complete.

**Boundary cases:** Some imported verified modules may contain trusted crypto or parser boundaries. Those must be counted and reported, not hidden.

### 5. Vendored seeds are manifest-gated and role-scoped

**Choice:** Optional imports from `../vericoder-mvp/`, `../verified-logic/`, and `../ucan/` require a seed manifest with source commit, paths, hashes, license/provenance review, role, and local acceptance command.

**Covers:** Scenario: Vericoder seed is accepted as architecture or trajectory fixture; Scenario: UCAN seed demonstrates core-shell vendoring shape; Scenario: Proof-relevant code cannot be imported as trusted by default

**Rationale:** These sibling repos contain useful patterns and code, but vendoring without provenance creates supply-chain ambiguity and review debt.

**Alternative considered:** Add path dependencies to sibling checkouts. That breaks reproducibility, Nix source isolation, and CI portability.

**Implementation:** Implement a `vendor-seeds` or standards fixture manifest format before any code import. Prefer fixtures/examples first; reusable library vendoring must have a stronger maintenance plan.

**Boundary cases:** Seeds can be advisory, fixture-only, reusable code, or proof-relevant. Each role has different acceptance evidence and must be explicit.

## Deterministic Proof Plan

- scenario or requirement:
  - Scenario: Lint-only policy remains explicit
  - Scenario: Target mode stronger than current mode records escalation
- artifact path:
  - `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- command / helper:
  - future policy validation command over positive/negative policy fixtures
- exact expected output / acceptance signal:
  - output names accepted `current`/`target` modes, rejects unknown modes, and reports missing escalation evidence for stronger target modes
- ordering / prerequisites:
  - policy schema implementation exists
- cleanup / postcondition:
  - fixtures remain checked in and referenced by the policy registry tests

- scenario or requirement:
  - Scenario: Closed Nickel policy contract rejects unknown fields
  - Scenario: Structural mode requires structural mechanism contract
  - Scenario: Verified mode requires proof rail contract
  - Scenario: Deny severity requires deterministic checker and evidence
- artifact path:
  - `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- command / helper:
  - future Nickel typecheck/export command over positive and negative policy fixtures
- exact expected output / acceptance signal:
  - valid fixtures export deterministic policy JSON/TOML; invalid fixtures fail with diagnostics for unknown fields, missing structural mechanisms, missing proof rails, and deny policies lacking checker/evidence contracts
- ordering / prerequisites:
  - `standards/policies/schema.ncl` and policy fixture files exist
- cleanup / postcondition:
  - exported artifacts are regenerated rather than hand-edited

- scenario or requirement:
  - Scenario: Crate boundary contract exports core and shell obligations
  - Scenario: Capability contract exports constructor authority
  - Scenario: Bounded and type-state contracts export API obligations
  - Scenario: Seed corpus contract exports provenance and role
- artifact path:
  - `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- command / helper:
  - future Nickel export command over mechanism-specific policy fixtures
- exact expected output / acceptance signal:
  - exported records include the mechanism fields required by Rust checkers for crate boundaries, capabilities, bounded primitives, type-state protocols, proof rails, and seed corpora
- ordering / prerequisites:
  - mechanism contracts exist in Nickel
- cleanup / postcondition:
  - Rust checker fixtures consume exported machine data rather than raw `.ncl` snippets

- scenario or requirement:
  - Scenario: No-std core boundary is enforced
  - Scenario: Shell-to-core dependency direction is enforced
  - Scenario: UCAN seed demonstrates core-shell vendoring shape
- artifact path:
  - `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- command / helper:
  - future structural checker over positive UCAN-style and negative ambient-effect fixtures
- exact expected output / acceptance signal:
  - positive fixture exits clean; negative fixture reports forbidden core effect and dependency-direction violation with policy IDs
- ordering / prerequisites:
  - fixture corpus and seed manifest exist
- cleanup / postcondition:
  - no dependency on sibling checkout remains after fixture generation

- scenario or requirement:
  - Scenario: Verified policy requires proof command
  - Scenario: Verified-logic seed requires local proof boundary
- artifact path:
  - `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- command / helper:
  - future proof-rail validation over verified seed manifest and local proof command transcript
- exact expected output / acceptance signal:
  - validation reports proof command, covered source/proof paths, pass signal, and trust-boundary count
- ordering / prerequisites:
  - verified seed manifest and local proof runner exist
- cleanup / postcondition:
  - proof-relevant seed is not marked accepted without local transcript

## Risks / Trade-offs

- Structural checks can be too rigid for stable crates -> use explicit compatibility profiles with negative fixtures.
- Vendoring can bloat the repo or blur ownership -> start with minimal fixtures/manifests and require local acceptance evidence.
- Verified mode can overpromise -> require trust-boundary accounting and keep sibling proof status advisory until locally reproduced.
- Mode-aware output can complicate existing JSON/SARIF consumers -> version machine fields carefully and preserve current lint finding shape where possible.
