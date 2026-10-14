# Trust-boundary and evidence path

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. It routes source through local checks, policy/config, and
reviewable artifacts before a team promotes the result into CI or release gates.

This page is an index and scope map. It links concrete Octet surfaces instead of
creating new claims.

## Source-to-gate flow

```text
Rust source
  -> cargo octet check / octet-standards checks
  -> checked-in policy and config (`dylint.toml`, `[workspace.metadata.octet]`, Nickel registry exports)
  -> evidence artifacts (`status.json`, SARIF, JSONL findings, external evidence receipts, transcripts, fingerprints, policy bundles)
  -> consumer CI or release gate
```

The same route applies to code written by agents and code written by humans. The
path is evidence-backed because every gate points to a command, policy file, or
artifact a reviewer can inspect.

## Evidence stack

The Octet evidence stack has four layers. Each layer names what it contributes
and where its boundary sits.

| Layer | Concrete Octet surfaces | Scope boundary |
| --- | --- | --- |
| discipline | Tiger Style guidance, closed-interface review rules, assertion and safety-critical guides | names review obligations and engineering discipline; it is not a substitute for domain review |
| static edge | Dylint lint domains, `cargo octet check`, SARIF and JSONL finding sidecars | rejects or reports known source shapes; lint cleanliness is not whole-program correctness |
| policy layer | Nickel-authored policy registry, `octet-policy-validate`, `octet-standards check`, generated policy bundles | validates declared policy/config contracts and source-shape gates; policy checks do not validate business requirements |
| evidence chain | `status.json`, `summary.txt`, transcripts, semantic fingerprints, policy artifact manifests, lifecycle evidence | preserves reviewable proof of what ran and what it inspected; evidence still needs human interpretation for product risk |

Formal proof artifacts, when present, stay explicitly scoped to their verifier,
model, receipt, and covered property. Other tiers remain validation or review
evidence and must not be upgraded by wording alone.

## Evidence tiers

Emitted evidence records use bounded, machine-readable tiers when the artifact
format carries tier data:

- `static_lint` — a Dylint or compiler lint finding from source analysis.
- `policy_check` — a standards or registry policy evaluation.
- `ui_fixture` — a checked UI fixture or diagnostic snapshot.
- `autofix_verified` — an autofix replay that rechecks cleanly.
- `transcript_assertion` — an executable transcript assertion with expected output.
- `formal_proof` — a scoped proof result with verifier, solver/toolchain, command argv, checked source/proof paths, receipt path, and proof artifact identity.
- `manual_review_required` — a boundary where a named human action is still required.

Validation is validation, review is review, and proof is proof for a stated model
and property. No tier may be silently promoted to a stronger claim.

## Checked-contract ladder

A checked contract in Octet is an evidence record, not a Rust syntax extension.
It is a scoped requirement, invariant, or review obligation that names what was
checked, how it was checked, where the artifact lives, and which claim remains
outside the evidence boundary.

The progressive assurance ladder keeps each stage explicit:

| Stage | Evidence tier or boundary | Command or gate | Artifact, receipt, or review surface |
| --- | --- | --- | --- |
| advisory lint findings | `static_lint` evidence for known source shapes | `cargo octet check` under the starter profile | stderr, SARIF, JSONL finding sidecars, and remediation text |
| warning gates | operational gating rather than formal proof; warning-level `static_lint` findings become review inputs | `cargo octet check` in CI or pre-commit with warning policy | `summary.txt`, `status.json`, SARIF, JSONL, and review notes |
| deny gates | operational gating rather than formal proof; denied lint findings block promotion | `cargo octet check` with deny profile or `templates/pre-commit-config.yaml` | non-zero runner status, sidecars, and suppression review |
| policy requirements | `policy_check` evidence tied to Nickel-authored policy IDs or selected external checker receipts | `octet-policy-validate`, `octet-standards check`, and `cargo octet evidence collect --rail ...` | generated policy bundles, standards diagnostics, dependency-audit, cargo-vet, semver, and release supply-chain receipts |
| receipt-backed validation | `static_lint`, `transcript_assertion`, `autofix_verified`, `ui_fixture`, dynamic profile, Kani model-check, Hegel property, or policy receipt evidence | Clippy co-reporting, transcript, dynamic evidence, Kani evidence, Hegel property evidence, autofix, UI, artifact, and standards validation gates | Clippy receipts, transcripts, dynamic profile receipts, Kani harness/contract/stub receipts, Hegel property receipts with bounded generated-testing non-claims, refreshed snapshots, verifier sidecars, and provenance records |
| scoped formal-proof receipts | `formal_proof` only for validated verifier-backed receipts | `cargo octet prove` followed by `cargo octet artifact verify` | structured proof receipt, proof card, bounded log identity, verifier identity, and provenance |
| verified release artifacts | release-gate evidence assembled from the lower tiers; artifact verification is not itself whole-program proof | `cargo octet artifact verify` and release/operator smoke transcripts | deterministic artifact directory, `provenance.jsonl`, smoke transcript, and reviewer signoff |

Every checked-contract record should include these fields:

- requirement or policy ID: the source ID for the obligation;
- source scope: crate, path, policy family, proof rail, or transcript section;
- invariant or failure shape: the invalid state, branch, missing guard, resource
  boundary, or remediation path the check can name deterministically;
- evidence tier: one of the bounded tiers above, or an explicit operational gate
  boundary when the stage is not formal proof;
- command or gate: the exact Octet command, CI gate, transcript, or release gate
  that produced the evidence;
- artifact or receipt path: the local artifact, receipt, proof card, manifest,
  transcript, or archive path a reviewer inspects;
- reviewer action: the human decision still required, such as suppression review,
  policy review, proof-card review, or release signoff;
- explicit non-claim: the unsupported conclusion the evidence does not prove.

Existing Octet surfaces already map to this shape. A lint diagnostic can bind a
requirement or lint ID to `static_lint` evidence. A standards policy can bind a
policy ID to `policy_check` evidence. Optional external evidence rails bind
`cargo-deny`, Clippy, `cargo-vet`, `cargo-semver-checks`, `cargo-mutants`,
`cargo-llvm-cov`, dynamic profile, and release supply-chain output to Rust-owned
receipts without making them starter checks. A Verus proof rail can bind a proof
card, receipt, provenance record, and artifact verifier command to a scoped
`formal_proof` claim. A `ProofEvidenceRef` can link a source comment, document,
or future suppression to that proof card by checking the referenced source file's
BLAKE3 identity and exact anchor text; it remains linkage evidence and does not
rerun Verus or emit `formal_proof`. Bounded evidence transforms are named,
rule-set-scoped normalization helpers with budgets, parent identities, and
non-claims; `octet.transform.claim-boundary-normalize.v1` and
`octet.rewrite.claim-boundary.v1` normalize claim-boundary text without becoming
unbounded tactics, solver completeness, semantic equivalence, or verified runtime
proof. Release evidence can bind artifact verification and a smoke transcript to
a reviewer action without upgrading every lower stage to proof.

High-confidence diagnostic examples should name the deterministic failure shape
when Octet has it. Examples include the invalid state behind an unchecked
narrowing, the branch where an unwrap lacks an adjacent guard, the missing guard
before division, the resource boundary for an unbounded channel, or the
remediation path for moving an ambient clock read into a shell. These examples
are not solver models and are not formal counterexamples unless the evidence
comes from a scoped verifier receipt that says so.

Checked-contract documentation does not introduce Rust `requires` or `ensures`
syntax, does not provide complete solver-backed verification for arbitrary Rust,
does not add runtime fallback checks for unproved contracts, and does not make
every ladder stage formal proof.

## Verus proof-path lane

Verus is an opt-in evidence lane inside this chain, not a replacement for the
other tiers. The canonical review route is:

```text
requirement or policy ID
  -> Verus proof rail
  -> checked Rust source scope
  -> proof target
  -> verifier/solver/toolchain identity
  -> structured proof receipt
  -> `formal_proof` provenance
  -> artifact verifier
```

Reviewers use the existing commands at the matching stage:

- `cargo octet prove --list` lists selected Verus proof rails without writing
  artifacts.
- `cargo octet prove --rail <policy-id[#rail-N]>` narrows a list, explain, or
  execution run to explicit rails after family/source matching; unmatched
  selectors fail closed with deterministic diagnostics.
- `cargo octet prove --explain` explains selected rails and deterministic
  remediation for stale bindings, verifier mismatches, unmatched rails, or
  unbacked proof claims.
- `cargo octet prove` runs the configured verifier command, writes bounded logs
  and structured receipts, and emits `formal_proof` provenance only for a
  validated scoped receipt.
- `cargo octet proof-ref scaffold` computes a consumer source BLAKE3 identity and
  emits linkage JSON copied from a saved proof card; `cargo octet proof-ref
  validate` checks that saved linkage without rerunning Verus or emitting new
  `formal_proof` provenance.
- `cargo octet artifact verify` recomputes artifact identities and checks saved
  receipts plus provenance before an artifact directory is treated as evidence.
  With `--emit-valence-provenance <path>`, it writes a Valence-ready export only
  after verification succeeds. r[impl octet.evidence_provenance.valence_export.boundary]
  r[impl octet.evidence_provenance.valence_export.boundary_hardened]
  The export is a Valence adapter input: every record carries a replayable
  receipt envelope, canonical BLAKE3 input/output artifact identities, typed edge
  kinds for checks, receipts, imported Trellis references, parent artifacts, and
  Valence adapter feeds, plus explicit non-claims. It preserves Octet artifact
  identities, provenance records, proof-card summaries, parent links,
  diagnostics, evidence tiers, and non-claims.
  r[verify octet.evidence_provenance.valence_export.hegel_receipts.boundary]
  Hegel property receipts export as scoped generated-testing facts with property
  IDs, linked requirement IDs, source scope, source hashes, parent links, and
  non-claim boundaries;
  r[verify octet.evidence_provenance.valence_export.not_release_claim]
  r[verify octet.evidence_provenance.valence_export.boundary.dependency_direction]
  the export does not prove release eligibility, Mantle bundle inclusion,
  whole-program correctness, verifier soundness, semantic equivalence,
  requirement satisfaction, or upgrade manual-review, lint, policy, Hegel, or
  imported Trellis evidence into formal proof.
- Proof goal-state feedback summarizes the selected claim, assumptions,
  satisfied evidence, missing obligations, blockers, remediation commands, and
  non-claims without requiring Lean execution or broadening proof claims.

The proof claim catalog at `standards/proof-claims/catalog.ncl` names stable
cross-surface proof/evidence claims, their source policy or requirement, checked
scope, expected evidence tier, allowed proof-card IDs, required artifacts,
documentation anchors, lifecycle status, aliases, owners, and scoped non-claims.
Catalog validation is a reference check: a catalog entry is not proof, and aliases
must be explicit before old names continue to resolve. Aliases are many-to-one
compatibility rows, not a bijection: several legacy or reviewer-facing names may
point to one canonical claim, while alias IDs must never conflict with canonical
claim IDs or other aliases.

Structured Verus proof package locks bind the selected rail manifest identity,
source/proof BLAKE3 bindings, command argv, environment allowlist, pass signal,
trust boundaries, verifier/solver/toolchain identity, closure graph identity when
available, and scoped non-claims before replay or promotion. Lock validation is
offline over checked-in/local artifacts; it does not fetch packages, schemas, or
network tool metadata and it does not prove verifier success by itself.

Structured Verus receipts name both the proof-result producer (`producer_id`, the
configured verifier identity) and the small deterministic checker (`checker_id`,
registered through policy `checker_ids`). Receipts also bind the proof package
lock BLAKE3 identity. A producer-only assertion, missing checker identity,
unregistered checker, stale lock, stale binding, or overbroad non-claim cannot
promote evidence to `formal_proof`. Starter `cargo octet check` remains
independent from proof package locks unless a proof rail is explicitly selected.

Imported Trellis references take a separate content-addressed branch:

```text
Trellis repository revision
  -> BLAKE3 source artifact identity
  -> BLAKE3 proof artifact identity
  -> bounded log identity when present
  -> receipt identity when present
  -> imported proof bundle
  -> proof card / manual review
  -> optional Octet rerun with pinned verifier identity
```

The current imported bundle manifest is Nickel-authored at
`standards/proof-imports/trellis.ncl` and exported to the deterministic runtime
JSON artifact `standards/proof-imports/generated/trellis-proof-imports.json`.
For that branch, BLAKE3 identifies exact artifact bytes and does not by itself
prove proof correctness. Imported Trellis bundles remain external or
`manual_review_required` until Octet reruns the referenced proof with pinned
verifier identity, validates an Octet-owned receipt, and then emits scoped
Octet provenance for that receipt only.

ChaosControl proof-style input contracts add a consumer-facing shape contract for
pinned Octet/Trellis proof-style gate inputs. The contract records the
ChaosControl consumer identity, selected Octet profile, focused crate list,
policy registry digest, optional Trellis primitive manifest digest, input
revision, expected command, and non-claims. Passing validation proves only that
those profile/input identities match Octet's declared contract; it does not run
ChaosControl VMM proofs, prove runtime determinism, prove ChaosControl
behavioral correctness, or transfer Octet lint/profile ownership to the
consumer.

The Verus lane preserves evidence-tier boundaries. Validated Verus proof receipts
may support `formal_proof`; lint findings remain `static_lint`, policy checks
remain `policy_check`, transcript assertions remain `transcript_assertion`,
imported Trellis bundles remain external or `manual_review_required` until
rerun, and unresolved human obligations remain `manual_review_required`.
Validation evidence stays validation evidence unless a selected verifier-backed
receipt supports the stronger tier.

The claim remains scoped to the declared property, checked source binding, proof
binding, verifier identity, receipt, and artifact identity. This lane does not
claim whole-program correctness, certification, business-domain validation,
semantic equivalence, full safety, or a verified runtime without separate
evidence.

## Provenance chain

A provenance record connects a claim to its source and artifact:

1. source ID: requirement, policy, lint, or transcript assertion;
2. checker ID: the tool or review surface that produced the evidence;
3. evidence tier: one of the bounded tier values above;
4. artifact locator: path or URI for the deterministic artifact;
5. BLAKE3 content hash: identity of the exact artifact bytes;
6. checked scope: source root, crate, policy family, or transcript section.

Release operators inspect the chain from requirement or policy ID to artifact,
then verify that the artifact hash, checker identity, tier, and scope match the
claim being made.

The initial deterministic producer registry covers `cargo-octet`,
`octet-standards`, `octet-policy-validate`, `octet-policy-export`,
`octet-fingerprint`, `octet-transcript`, `cargo-deny`, Clippy, `cargo-vet`,
`cargo-semver-checks`, `cargo-mutants`, `cargo-llvm-cov`, `cargo-kani`,
`hegel-rs`, `octet-dynamic-evidence`, and `octet-release-supply-chain`. Source
IDs stay in their producer families (`octet.runner.*`, `octet.artifact.*`,
`octet.standards.*`, `octet.policy.*`, `octet.external_lint.*`,
`octet.dependency_trust.*`, `octet.api_stability.*`,
`octet.mutation_testing.*`, `octet.coverage_baseline.*`,
`octet.kani_model_check.*`, `octet.dynamic_evidence.hegel-property`,
`octet.dynamic_evidence.*`, `octet.release_supply_chain.*`,
`octet.fingerprint.*`, and `octet.transcript.*`), and parent artifacts link
derived evidence back to the runner, policy, fingerprint, transcript, or release
artifact it summarizes. Missing parents and parent cycles are rejected;
validation and review artifacts remain `static_lint`, `policy_check`,
`transcript_assertion`, or `manual_review_required`, not `formal_proof`.

## What Octet does not claim

Octet is useful because its boundaries are explicit:

- Octet does not prove whole-program correctness. It does provide static
  rejection of known-dangerous source shapes, policy validation, and artifacts
  such as SARIF, JSONL findings, transcripts, and fingerprints.
- Octet does not validate business or domain requirements. It does preserve
  requirement and policy IDs so reviewers can trace which engineering claim an
  artifact supports.
- Octet does not make suppressions correct merely because they are present. It
  does require reviewable suppressions, narrow reasons, and follow-up gates for
  teams that opt into those checks.

A clean run is an input to review, not a complete assurance argument.

## Controlled deployment posture

Octet is designed for local or consumer-CI execution:

- local execution: `cargo octet check` runs in the consumer workspace;
- consumer CI execution: Nix and Cargo entry points can run under the team's own
  CI boundary;
- no required hosted service: Octet does not require uploading source or
  artifacts to an Octet-operated service;
- pin-capable runner path: consumers can pin `github:OnixResearch/octet` to a
  release tag or commit, or use `path:/path/to/octet#cargo-octet`;
- source-visible toolchain: this repository contains the runner, lint library,
  standards checker, policy registry, and generated artifact seam;
- local artifact directory: `cargo octet check --artifact-dir artifacts/octet`
  writes review files plus `provenance.jsonl` to a configured local path, and
  `cargo octet artifact verify --artifact-dir artifacts/octet` checks the saved
  bundle without mutating it.

Boundary caveat: Octet itself does not require uploading source, logs, or
artifacts. Consumers still own what their CI provider, log collector, artifact
store, or downstream integrations do after Octet writes local files.

### Regulated-consumer checklist

- Pin the Octet revision or local path used by `nix run ...#cargo-octet` or the
  pre-commit hook `rev`.
- Check in consumer-owned `dylint.toml` and `[workspace.metadata.octet]` runner
  scope.
- Configure a deterministic artifact directory such as `artifacts/octet` for
  `command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`, and structured output, then run the artifact verifier before promotion.
- Add optional external evidence directories only when selected, for example `artifacts/dependency-audit` or `artifacts/release-supply-chain`, and verify each receipt bundle separately.
- Optionally add `templates/pre-commit-config.yaml` as a deny-all pre-commit
  gate after the starter profile is understood.
- Save or reference the release/operator smoke command when promoting the gate.

## Trust-boundary and evidence-path index

| Theme | Where to start | Concrete surface |
| --- | --- | --- |
| checked-path framing | [README quick start](../README.md#quick-start) and [consumer guide](consumer-guide.md) | `cargo octet check`, `octet-standards check`, source-to-gate flow |
| evidence stack | [Evidence stack](#evidence-stack) | discipline, static edge, policy layer, evidence chain |
| evidence tiers | [Evidence tiers](#evidence-tiers) | `evidence_tier` fields in JSONL/SARIF sidecars and provenance DTOs |
| checked-contract ladder | [Checked-contract ladder](#checked-contract-ladder) | advisory lint findings, warning gates, deny gates, policy requirements, receipt-backed validation, scoped formal-proof receipts, and verified release artifacts |
| provenance chain | [Provenance chain](#provenance-chain) | `provenance.jsonl` records source ID, checker ID, tier, artifact locator, BLAKE3 hash, and checked scope; `cargo octet artifact verify` recomputes and checks saved bundles |
| bounded transforms | [Bounded evidence transforms](bounded-evidence-transforms.md) | named rule-set-scoped transforms with budgets, parent identities, output identities, diagnostics, and explicit non-claims |
| proof goal-state feedback | [Proof goal-state feedback](proof-goal-state-feedback.md) | deterministic obligation view for claims, assumptions, satisfied evidence, missing obligations, blockers, remediation commands, and non-claims |
| type-driven domain modeling | [Type-driven domain modeling](type-driven-domain-modeling.md) | small Rust domain identities, enums, and refined constructors protecting core/adapters while preserving wire fields |
| effect capability ports | [Effect capability ports](effect-capability-ports.md) | pure cores return process/effect plans and diagnostics; shells own filesystem, process, environment, time, randomness, logging, metrics, network, and runtime effects |
| scope-boundary | [What Octet does not claim](#what-octet-does-not-claim) and [safety-critical guide](safety-critical.md) | unsupported claims paired with supported capabilities |
| controlled-deployment | [Controlled deployment posture](#controlled-deployment-posture) and [downstream rollout kit](downstream-rollout.md) | local/CI execution, pinned runner, source-visible toolchain, local artifacts |
