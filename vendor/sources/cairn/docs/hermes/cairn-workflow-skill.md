---
name: cairn-workflow
description: >
  Work on the Cairn Rust/Nix lifecycle engine in /home/brittonr/git/cairn:
  native Cairn specs and archives, MCP tools, deterministic receipts,
  release-readiness evidence, Cairn-owned traceability coverage, Slotcar FCIS checks, and Nix
  verification. Use for Cairn CLI/core/spec/test/flake/docs work.
---

# Cairn Workflow

<!-- r[verify cairn.supersede.hermes_skill] r[verify cairn.supersede.hermes_skill.discover] -->
<!-- r[verify cairn.supersede.hermes_skill.canonical_cairn] r[verify cairn.supersede.hermes_skill.canonical_cairn.path] -->
<!-- r[verify cairn.supersede.hermes_skill.canonical_cairn.live] r[verify cairn.supersede.hermes_skill.redirect] -->
<!-- r[verify cairn.supersede.hermes_skill.redirect.local] r[verify cairn.supersede.hermes_skill.contracts] r[verify cairn.supersede.hermes_skill.audit] -->

Use this Hermes skill for work in `/home/brittonr/git/cairn`. Cairn is the canonical product, repository, CLI, package, lifecycle layout, and skill name. For normal agent workflow, reason only in native Cairn terms; compatibility bridges are explicit product features, not alternate planning authority.

## Project shape

- Rust workspace with Nix flake workflow.
- CLI shell: `crates/cairn-cli`.
- Pure deterministic core: `crates/cairn-core`.
- Native lifecycle content lives under `.cairn/`.
- Legacy lifecycle content is an explicit import/migration source, not a runtime fallback layout.
- Fixtures live under `fixtures/` and are part of the behavior contract.
- Slotcar input MUST stay `github:OnixResearch/slotcar`.
- Run the CLI through Nix as `nix run path:$PWD#cairn -- ...`.

## Operating mode

- For public CLI behavior, compatibility semantics, mutation, architecture, receipt schemas, or roadmap work, write a native `.cairn/changes/<slug>/` package first.
- Small fixture/test/doc hardening can be direct when it does not alter public behavior.
- Prefer one narrow ROI slice at a time, then verify, sync/archive the change when applicable, commit, and integrate it.
- Treat compatibility surfaces as import-only product behavior; native Cairn remains the sole planning authority.

## Worktree and branch completion

Complete every Cairn change through an isolated worktree and unique new branch. This rule applies when the primary worktree is clean or dirty.

1. Fetch `origin/main` and create the new branch from it in a dedicated worktree.
2. If the change package is uncommitted, copy only its inspected files into the worktree.
3. Leave the primary worktree files unchanged.
4. Implement, validate, sync, archive, and commit the change only in the dedicated worktree.
5. Push the verified change branch to `origin`.
6. Fetch `origin` again before integration.
7. If `origin/main` advanced, merge it into the change branch and rerun the relevant checks.
8. Push the updated branch, fast-forward `main` to it, and push `main` to `origin`.
9. Verify that `origin/main` contains the completion commit. Then remove the worktree.

A request to complete or drain Cairn changes authorizes these normal change-branch and `main` pushes.
Never open a pull request or force-push. Preserve unrelated work and report any integration blocker.

## Native change package workflow

Create changes with:

```bash
nix run path:$PWD#cairn -- change create <slug> --profile spec-driven --root .
```

The available profiles are `spec-driven`, `assurance`, and `no-spec-delta`. Existing changes without profile metadata use `spec-driven`.

`assurance` adds risk, verification, and current semantic-review evidence. `no-spec-delta` uses local `a[...]` criteria and an explicit no-op sync.

A complete `spec-driven` change normally includes:

- `proposal.md`
- `design.md`
- `tasks.md`
- `specs/<accepted-spec>/spec.md`

Validate the package before implementation or committing spec-only work:

```bash
nix run path:$PWD#cairn -- validate --root .
nix run path:$PWD#cairn -- gate proposal <slug> --root .
nix run path:$PWD#cairn -- gate design <slug> --root .
nix run path:$PWD#cairn -- gate tasks <slug> --root .
```

Replace every generated `CAIRN_PLACEHOLDER` before gating. Requirement blocks need exactly one bare `r[...]` ID and normative requirement-body prose. Checkbox tasks need a substantive label plus a delta requirement reference or a structurally valid dependency marker; checked state does not override substance failures. Cross-spec refs to already-accepted IDs can fail the tasks gate.

## FCIS boundary

- `cairn-core` is the functional core: pure deterministic transforms only.
- Keep pure code under `crates/cairn-core/src/verified/` so Slotcar checks cover it.
- Filesystem, environment, process execution, stdout/stderr, mutation, probe execution, and JSON presentation belong in `crates/cairn-cli`.
- Structured lifecycle source uses Nickel contracts plus generated JSON bundles; Markdown is a render/import projection once structured source exists.
- Default gates do not execute probe commands. Run `cairn probe run ... --out ...` first, then let gates verify saved probe receipts; `--run-probes` is explicit and denied by default until policy enables a bounded profile.
- Do not add filesystem traversal, environment reads, process execution, clocks, async, `println!`, `eprintln!`, or JSON rendering helpers to core.

<!-- r[impl cairn.agent_guidance_context.docs.authority] -->

Core purity smoke check:

```bash
rg 'std::fs|fs::|std::env|std::process|println!|eprintln!|read_dir|read_to_string|to_json|std::time|tokio::|async' crates/cairn-core/src
```

Expected result: no matches.

## Current command contracts

- Native lifecycle/read commands:
  - `cairn init --root PATH`
  - `cairn change create <name> [--profile spec-driven|assurance|no-spec-delta] --root PATH`
  - `cairn change depend list <name> --root PATH`
  - `cairn change depend add <name> <dependency> --root PATH [--execute]`
  - `cairn change depend remove <name> <dependency> --root PATH [--execute]`
  - `cairn change list --root PATH`
  - `cairn change graph --root PATH [--format json|mermaid|dot]`
  - `cairn change next --root PATH`
  - `cairn change show <name> --root PATH`
  - `cairn view kanban|mindmap|graph --root PATH [--format json|markdown|mermaid|dot] [--interactive]`
  - lifecycle views are read-only projections over one deterministic semantic model; JSON is the automation default, Subwayrat-backed graph canvases are opt-in display shells, and visual layout state is excluded from receipt hashes.
  - `cairn spec list --root PATH`
  - `cairn spec show <id> --root PATH`
  - `cairn workflow status <change> --root PATH [--schema ID]`
  - `cairn workflow instructions <artifact> --change <change> --root PATH [--schema ID]`
  - `cairn context --root PATH [--profile ID] [--artifact ID] [--operation ID]`
  - Use only checked project guidance from Cairn results. Do not scan for ambient prompt or instruction files.
  - Keep `policy_constraint`, `project_fact`, and `advisory_guidance` authority classes separate.
  - Advisory guidance cannot satisfy gates or authorize mutation. Portable guidance evidence omits raw text and absolute paths.
  - Workflow commands report the immutable selected profile and bounded structural facts. `--schema` is an assertion, not a profile override.
  - Profile conformance does not prove zero behavior effect, risk-analysis truth, test sufficiency, code correctness, or release eligibility.
  - local `depends_on` ordering dependencies are authored through preview-by-default `change depend add/remove`; archived dependencies satisfy blockers, active dependencies block sync/archive, and cross-repo receipt/evidence dependencies remain a separate evidence-input concept.
  - Native `.cairn/` content is used for normal runtime commands. Old `cairn/`, `ironspec/`, and `openspec/` content is migration-only.
- Diagnostics:
  - `cairn doctor --root PATH [--policy PATH]`
  - read-only receipt fields include `valid`, `layout`, spec/change counts, validation issue count, and named checks.
  - policy read/parse failures are reported as JSON checks rather than process failures.
- Validation:
  - `cairn validate --root PATH`
  - receipt fields include machine-readable `findings`, `spec_findings`, and `substance_findings` alongside compatibility `issues`, `change_issues`, `spec_issues`, `spec_substance`, `substance_issues`, `substance`, and `specs_validated`.
  - accepted specs and active delta specs are checked for EARS-style quality and policy-backed structural substance.
- Gate policy:
  - `cairn gate proposal <change> --root PATH`
  - `cairn gate design <change> --root PATH`
  - `cairn gate tasks <change> --root PATH`
  - `cairn gate tasks <change> --root PATH --run-probes` is denied by default unless policy later enables inline probe execution.
  - stable public receipt fields are `change`, `stage`, `verdict`, `valid`, `issues`, `substance`, `policy`, `layout`, `policy_hash`, `input_hash`, and `receipt_hash`; each gate issue includes a stable `code`.
  - AgentKit/OpenGate is a policy-shape reference only; Cairn owns the Nickel policy under `cairn-policy/` and does not depend on AgentKit at runtime.
  - Nickel owns generated receipt/action/hash schema declarations, marker vocabulary, exemption marker required/date fields, and semantic policy invariants. Runtime Rust consumes `cairn-policy/generated/cairn-policy.json`; do not add runtime Nickel execution.
  - Gates read proposal/design/tasks/delta specs in the CLI, pass a pure snapshot to core, and fail on unknown `r[...]` references, unlinked tasks at the tasks gate, or malformed `gate-exempt[...]` markers that do not match the generated exemption marker contract.
  - Task gates also observe bounded saved probe declarations/receipts without execution. Only current explicit passing evidence satisfies a declaration; the final issue set derives one coherent verdict and blocks CLI/MCP sync/archive mutation.
- Sync/archive:
  - `sync <change>` and `archive <change>` are dry-run by default.
  - `--execute` is required for mutation.
  - executed sync merges explicit ADDED/MODIFIED/REMOVED requirement sections into accepted specs by stable `r[...]` ID and preserves accepted title/purpose/`## Requirements` wrapper content; full-spec-shaped, empty, ambiguous, conflicting, or malformed deltas fail closed before any write.
  - stable public plan receipt fields are `actions`, `blocked`, `change`, `dry_run`, `input_hash`, `layout`, `mutated`, `plan_hash`, `policy`, `policy_hash`, `reasons`, and `receipt_hash`; sync receipts also include `delta_specs` and `merge_preflight` source/pre-state/operation/result bindings.
  - sync/archive action entries expose stable `kind`, `path`, and `description` fields.
  - `--execute` refuses mutation when the configured gate adds blockers to an otherwise unblocked plan.
  - blocked plans must not mutate, even with `--execute`.
  - For real archives, set `CAIRN_ARCHIVE_DATE=$(date +%F)` before `cairn archive <change> --execute`.
- Lossless archive compaction:
  - `cairn compact archive --root PATH --profile lossless-pack --out FILE` emits a deterministic saved plan without mutating lifecycle storage.
  - `cairn compact archive --root PATH --plan FILE --execute` rechecks the saved plan, writes and verifies content-addressed pack artifacts, appends the receipt chain, then prunes expanded archives.
  - `cairn compact verify --root PATH` checks ledger continuity, manifests, objects, pointers, mutation linkage, and interrupted/orphaned artifacts.
  - `cairn compact unpack --root PATH --archive ID [--execute]` previews or restores exact archived bytes; only the lossless profile is executable.
  - Merkle-style roots bind represented content; chain hashes bind observed operation order. Passing verification proves local representation integrity only, not archived correctness, release readiness, remote trust, or semantic review completeness.
- Structured lifecycle/probes:
  - `cairn lifecycle check --source PATH`
  - `cairn lifecycle export --source PATH --output PATH [--check]`
  - `cairn lifecycle render-markdown --source PATH [--out-dir DIR]`
  - `cairn lifecycle import-markdown --root PATH --out PATH [--execute]`
  - `cairn probe run --root PATH --probe-file PATH [--out PATH]`
  - `cairn probe validate --root PATH --probe-file PATH --receipt PATH`
- Change collaboration:
  - The semantic state is backend-neutral: workspace and typed base-revision identities, exact artifacts, conflicts, lifecycle, and evidence reduce independently of provider object IDs.
  - `cairn collaboration import <change> --backend portable-json.v1 --store PATH --workspace ID --base-revision KIND:VALUE --actor ID --root PATH [--policy PATH] [--execute]` creates a validated portable state without Radicle; actor/workspace inputs are declarations, not cryptographic authority.
  - `cairn collaboration status <change> --backend portable-json.v1 --store PATH --root PATH [--policy PATH]` is bounded and read-only; `collaboration checkout ... [--execute]` is preview-first and refuses conflicts, unsafe stores, changed state, and divergent local work.
  - `cairn collaboration update <change> --backend portable-json.v1 --store PATH --actor ID --root PATH [--policy PATH] [--execute]` derives sorted exact artifact actions with complete current revision bases, treats byte-equivalent state as a no-op, and rechecks state/policy/local bytes under serialized execution before validated atomic replacement.
  - `cairn collaboration conflicts <change> --backend portable-json.v1 --store PATH --root PATH [--show-head ARTIFACT=REVISION]` reports sorted metadata for every current artifact/lifecycle conflict and discloses at most one explicitly selected exact artifact value. Conflicted portable envelopes use `content_root: null` while retaining validated state and envelope hashes.
  - `cairn collaboration resolve <change> --backend portable-json.v1 --store PATH --actor ID --choose-head ARTIFACT=REVISION... --root PATH [--lifecycle open] [--execute]` requires exactly one current-head choice for every conflicted artifact, creates new revisions whose basis supersedes every current head, and resolves lifecycle conflicts only through an explicit evidence-free `open` reset.
  - `--backend radicle-cob.v1` routes neutral commands to the COB-only adapter over required `--git-repository PATH`; import, update, and resolution additionally require `--keystore PATH`, and import requires `--workspace ID` with optional repeated `--delegate PUBLIC_KEY`. `cairn cob import|status|checkout|update|conflicts|resolve` is the direct entry point.
  - The adapter uses `radicle-cob` for standard signed Git commits, namespaced refs, object/type IDs, and operation-DAG evaluation. The explicit repository must have repository-local `user.name` and `user.email` for non-authoritative commit display metadata; global Git config is not a fallback. It does not load profiles or identity documents and has no Radicle home, RID, node, discovery, seeding, follow, announce, control-socket, peer, connect/fetch, sync, or convergence behavior.
  - Neutral roots embed immutable creator/delegate authority. Signed mutations recheck the explicit keystore actor against that embedded set under a per-change Git lock, reload typed state, and verify the planned content root; caller-supplied delegate claims cannot add authority. Existing legacy objects remain readable with exact historical hashes and creator-only mutation.
  - `nix build path:$PWD#checks.x86_64-linux.radicle-cob-format-compatibility --no-link -L` validates signed standard COB refs/commits, malformed-history rejection, a typed BLAKE3-bound static bundle from upstream Radicle Git storage, Cairn append compatibility, a separate direct read-only consumer, and the absence of umbrella `radicle` or `radicle-node` product dependencies. The standalone fixture producer is provenance evidence only, not a workspace member, Nix input, released executable, or runtime dependency.
  - Ordinary update refuses content and lifecycle conflicts. Explicit resolution never uses local bytes as an implicit winner, rewrites history, permits partial choices, carries approval-like lifecycle state forward, or claims semantic merge correctness.
  - Portable and COB locks serialize cooperating local Cairn processes. Stale locks/prepared state require deliberate recovery; no backend receipt proves losing-head invalidity, lifecycle approval, evidence truth, network publication, synchronization, peer convergence, future authority, archive execution, release eligibility, or resistance to unrelated processes with equivalent host authority.
- Migration:
  - Cairn migration commands emit dry-run copy plans from old lifecycle layouts to native `.cairn/`.
  - Use `cairn migrate cairn` for the old visible directory.
  - Review the preview `plan_hash`. Pass that hash with `--execute --plan-hash HASH`.
  - Execution copies accepted specs, active changes, and archives. It never deletes old content.
  - Source, destination, action, or policy drift blocks execution.
  - Existing native destinations are blockers and must not be overwritten.
  - stable public migration receipt fields are `command`, `root`, `source_layout`, `destination_layout`, `actions`, `conflicts`, `blocked`, `dry_run`, `mutated`, `policy`, `policy_hash`, `source_manifest_hash`, `destination_manifest_hash`, `manifest_hash`, and `receipt_hash`.
  - migration action entries expose stable `source`, `destination`, `source_hash`, and `destination_hash` fields.
- MCP and agent receipts:
  - `cairn mcp --describe`
  - `cairn mcp --call <tool> '<json>'`
  - `cairn mcp --allow-mutations` is required for mutation-capable tools.
  - `cairn_change_graph` is the read-only MCP change dependency graph receipt: it reports dependency declarations, blockers, reverse `blocks`, graph issues, and dependency-first drain order.
  - `cairn_drain_plan` is the read-only MCP repository-wide drain planner: it summarizes every active change with gate/task readiness, dependency blockers, and safe next actions; ready changes still require explicit `cairn_sync_execute` / `cairn_archive_execute` calls under `--allow-mutations`.
  - `cairn_workflow_status` returns policy-owned artifact states, blockers, unlocks, and next actions.
  - `cairn_workflow_instructions` returns one policy-owned instruction envelope with upstream BLAKE3 identities.
  - Both workflow tools are read-only. Their results do not prove approval, correctness, gate passage, or release eligibility.
  - MCP tool schemas are strict object schemas with `additionalProperties=false`.
  - Default mutation denial is checked before argument validation for mutation-capable tools.
  - `cairn agent-receipt validate <path>` validates deterministic agent-run receipt fixtures.
  - `cairn agent-receipt verify-bundle <bundle.json> --root .` validates a persisted agent/CI evidence chain across an agent receipt, saved replay receipt, saved replay verification receipt, and optional release-readiness receipt.
  - `cairn evidence-manifest generate --bundle <bundle.json> --agent-receipt <agent.json> --replay <replay.json> --replay-verification <verify.json> [--registry <registry.json>] [--registry-verification <verify.json>] [--registry-selection <select.json>] [--registry-fetch-plan <plan.json>] --out <manifest.json> --root .` generates canonical outer manifest JSON with stable roles and artifact hashes, including registry publish/consume evidence when provided.
  - `cairn evidence-manifest verify <manifest.json> --root .` validates the outer persisted evidence artifact index, declared hashes, supported roles, and optional public signer/anchor metadata without claiming cryptographic signature or remote-anchor validity.
  - `cairn evidence export|verify|import|republish|index` builds the deterministic evidence chain from bundle materialization through publishable single-bundle indexes.
  - `cairn evidence registry build --root . --index <index.json> --out <registry.json> [--channel stable] [--release YYYY-MM-DD] [--platform x86_64-linux] [--compatibility-class current]` and `cairn evidence registry verify --root . --registry <registry.json>` aggregate and verify deterministic multi-index release/channel catalogs with policy-validated stable labels.
  - `cairn evidence registry mirror-verify --root . --registry <registry.json> --mirror <dir>` verifies offline/mirrored registry indexes via safe `index_relative` refs, preserving labels/receipt chains and emitting portable mirror receipts without copying files or network I/O.
  - `cairn evidence registry select --root . --registry <registry.json> --channel stable --bundle-kind <kind> --out <selected-index.json>` copies exactly one selected index after registry verification; `cairn evidence registry fetch-plan --root . --registry <registry.json> --channel stable --out <fetch-plan.json>` emits metadata-only fetch/verify plans with safe label metadata; `cairn evidence registry fetch-execute --root . --plan <fetch-plan.json> --source <local-mirror-dir> --out <artifact-cache-dir>` locally materializes planned artifacts with safe relative paths and BLAKE3 verification before/after copying, without network transport, credential handling, or remote trust validation.
  - `cairn agent-receipt smoke-mcp --root .` proves the MCP catalog, receipt validation path, strict invalid-argument behavior, and default mutation denial compose.
- Release readiness:
  - `cairn release-readiness --root .` emits a deterministic local operator bundle over Cairn validation, Cairn-owned traceability coverage, MCP smoke, repository gate summary, and receipt hashes.
  - Release-readiness receipts should be byte-for-byte reproducible for the same root.

## EARS-style validation contract

- Requirements need normative strength words such as `MUST`, `SHALL`, `SHOULD`, or `MAY`.
- Every `#### Scenario:` block needs `GIVEN`, `WHEN`, and `THEN` clauses.
- Keep validation decisions in pure core over in-memory `SpecSource` markdown; CLI owns file reads and JSON receipts.

## Verification rails

Run the full gate after substantive changes:

```bash
nix run path:$PWD#cairn -- policy export --check
nix develop -c cargo fmt --check
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix run path:$PWD#cairn -- validate --root .
nix run path:$PWD#cairn -- traceability coverage --root .
nix flake check "path:$PWD" -L
```

Useful focused checks:

```bash
nix run "path:$PWD#cairn" -- validate --root .
nix run "path:$PWD#cairn" -- release-readiness --root .
nix build "path:$PWD#checks.x86_64-linux.self-dogfood" --no-link -L
nix build "path:$PWD#checks.x86_64-linux.tracey-coverage" --no-link -L
nix build "path:$PWD#checks.x86_64-linux.slotcar" --no-link -L
```

Remote builder warnings for `ssh-ng://root@10.10.10.1` are non-blocking if Nix continues locally and the requested checks pass.

## Traceability coverage rail

- Requirement markers in accepted specs use `r[...]`; implementation and test comments should use `r[impl ...]` and `r[verify ...]`.
- `cairn traceability coverage --root .` is the deterministic repo-owned rail used by `nix flake check`; it loads Nickel-authored scan profiles from generated policy and ensures every accepted requirement has at least one source-controlled implementation or verification reference.
- `cairn tracey coverage --root .` remains a compatibility alias only. External `tracey query ...` commands are optional developer diagnostics and are not required by maintained gates, release-readiness evidence, or Nix checks.
- Marker coverage does not prove behavioral correctness; cite executable tests, proofs, or stronger receipts for behavioral claims.

## Known pitfalls

- Direct host `cargo` may be unavailable; use `nix develop -c ...`.
- Nix flakes snapshot tracked/staged files. Stage new source/doc files before `nix flake check` if the check needs them.
- `cairn sync <change>` requires explicit `## ADDED Requirements`, `## MODIFIED Requirements`, or `## REMOVED Requirements` operations with one stable bare requirement ID per block. If `merge_preflight` reports blockers, repair the delta shape or identity conflict and rerun the dry-run; never substitute a manual post-sync accepted-spec patch for a blocked semantic plan. See `docs/sync-delta-merge.md`.
- Historical archives and compatibility code may mention legacy lifecycle names; do not churn those unless the task is explicitly a migration/rename cleanup.
- Slotcar Nix checks need a C compiler/linker; keep `pkgs.stdenv.cc` in check inputs.
- The canonical repo uses `git@github.com:OnixResearch/cairn.git`. Complete verified slices through the required worktree, branch, and `main` integration workflow.

## Skill maintenance

- Update this source when Cairn changes workflow contracts, verification rails, or command semantics.
- Keep it class-level and reusable; do not add transient session progress or credentials.
- Use this skill for Cairn work instead of legacy or generic lifecycle skills, except when the task is specifically about compatibility behavior.
