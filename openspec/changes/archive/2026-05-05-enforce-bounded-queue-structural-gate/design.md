## Context

The policy registry already models structural enforcement modes and generated artifacts. `tigerstyle.embedded.bounded-queues` is selected by Aspen through `lib.mkPolicyArtifactConsumerCheck`, but the current standards checker only has a narrow textual resource scan and the downstream proof only asserts zero diagnostics plus policy metadata. This change turns that seam into a concrete structural gate for bounded queue construction.

## Goals / Non-Goals

**Goals:**
- Make the bounded queue policy artifact carry enough mechanism data for Rust checker enforcement.
- Reject configured unbounded constructors with diagnostics that include the policy ID, structural mechanism, failure kind, source path, line, and remediation.
- Accept bounded constructors or approved wrappers only when an explicit capacity or backpressure witness is present.
- Prove the downstream Aspen check fails under a scratch mutation that introduces one configured unbounded constructor.

**Non-Goals:**
- Replacing all consumer queue APIs with a new Tigerstyle crate in this change.
- Solving whole-program queue capacity dataflow beyond declared local/wrapper witnesses.
- Changing the Dylint `unbounded_channel` lint semantics except where shared fixtures/docs need alignment.

## Decisions

### 1. Drive the checker from the bounded primitive mechanism export

**Choice:** Extend the Nickel bounded primitive mechanism and generated artifacts so `tigerstyle-standards check --policy-artifacts-dir` reads forbidden APIs, approved APIs, wrapper scopes, and witness requirements from the policy artifact.

**Covers:** Scenario: Bounded queue artifact exports checker obligations; Scenario: Artifact-backed checker rejects configured unbounded constructors

**Rationale:** Structural enforcement should consume the policy artifact, not hard-code intent in Rust or rely on prose.

**Alternative considered:** Keep the current static string list in `resource.rs`. That cannot prove the generated artifact seam owns enforcement and makes downstream policy drift invisible.

**Implementation:** Add mechanism fields in `standards/policies/contracts.ncl` / `registry.ncl`, preserve them in export artifacts, and make `resource.rs` build matchers from the loaded `PolicyEntry.mechanisms`.

**Boundary cases:** Missing mechanism fields for a structural bounded primitive policy remain a policy-contract error; lint-only `unbounded_channel` behavior remains separate from the structural artifact gate.

### 2. Treat explicit capacity and backpressure as the accepted construction witness

**Choice:** Accept bounded queue usage when the constructor or wrapper exposes an explicit capacity argument/constant and a configured witness records overflow or backpressure behavior.

**Covers:** Scenario: Bounded construction with witness passes; Scenario: Missing capacity witness remains a structural finding

**Rationale:** A bounded constructor without a reviewable capacity/backpressure story is weaker than the policy promise.

**Alternative considered:** Accept any bounded API call such as `mpsc::channel(_)`. That misses hidden defaults or wrapper constructors that drop capacity evidence.

**Implementation:** Fixtures cover accepted direct bounded Tokio/Crossbeam/std-like patterns, approved wrapper modules, and missing-witness cases. The first implementation may use precise local syntactic witnesses before adding semantic Rust analysis.

**Boundary cases:** Non-local dataflow that cannot prove capacity should fail closed unless the wrapper/module is explicitly approved by the mechanism contract.

### 3. Prove consumer enforcement with an Aspen scratch mutation

**Choice:** Preserve the existing clean Aspen check, then create a temporary scratch copy or worktree mutation that adds exactly one configured unbounded constructor in a checked Aspen root and verifies `checks.x86_64-linux.tigerstyle-policy-artifacts` fails for `tigerstyle.embedded.bounded-queues`.

**Covers:** Scenario: Downstream mutation fails the policy artifact consumer check

**Rationale:** The new helper is only valuable if real consumers cannot silently drift. Local fixtures alone do not prove the consumer flake wiring.

**Alternative considered:** Document the helper without mutation proof. That repeats metadata dogfood and does not demonstrate enforcement.

**Implementation:** Capture the clean check and negative scratch transcript under the change evidence directory. The mutation must be reverted or confined to a temp copy before final status.

**Boundary cases:** Aspen may have unrelated flake eval debt; the evidence command should target `checks.x86_64-linux.tigerstyle-policy-artifacts` and classify unrelated checks as out of scope.

## Deterministic Proof Plan

- scenario or requirement:
  - Scenario: Bounded queue artifact exports checker obligations
- artifact path:
  - `openspec/changes/archive/2026-05-05-enforce-bounded-queue-structural-gate/evidence/policy-artifact-export.txt`
- command / helper:
  - `nix run .#tigerstyle-policy-artifacts -- /tmp/tigerstyle-policy-artifacts-bounded-queue-proof` plus JSON/TOML assertions over the exported bundle
- exact expected output / acceptance signal:
  - exported artifacts include `tigerstyle.embedded.bounded-queues`, forbidden unbounded APIs, approved bounded APIs or wrapper scopes, capacity-witness requirements, and backpressure or overflow behavior requirements
- ordering / prerequisites:
  - mechanism contract and registry updates are implemented before this export proof is captured
- cleanup / postcondition:
  - `/tmp/tigerstyle-policy-artifacts-bounded-queue-proof` is temporary and not committed

- scenario or requirement:
  - Scenario: Artifact-backed checker rejects configured unbounded constructors
  - Scenario: Bounded construction with witness passes
  - Scenario: Missing capacity witness remains a structural finding
- artifact path:
  - `openspec/changes/archive/2026-05-05-enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- command / helper:
  - `nix build .#checks.x86_64-linux.standards-fixtures --no-link -L`
- exact expected output / acceptance signal:
  - exits successfully for compliant fixtures and fails internally if expected bounded-queue policy diagnostics are absent from noncompliant fixtures or if compliant witness fixtures report diagnostics
- ordering / prerequisites:
  - mechanism fields, checker consumption, and fixture manifest updates are implemented before this check is captured
- cleanup / postcondition:
  - no generated temp artifact directories are committed

- scenario or requirement:
  - Scenario: Downstream mutation fails the policy artifact consumer check
- artifact path:
  - `openspec/changes/archive/2026-05-05-enforce-bounded-queue-structural-gate/evidence/aspen-policy-artifact-mutation.txt`
- command / helper:
  - `nix build .#checks.x86_64-linux.tigerstyle-policy-artifacts --no-link -L` from an Aspen scratch copy containing one inserted unbounded constructor
- exact expected output / acceptance signal:
  - exits non-zero and the saved JSON/log names `tigerstyle.embedded.bounded-queues`, the inserted file, and exactly one expected mutation diagnostic
- ordering / prerequisites:
  - local standards fixtures and artifact consumer fixture pass first
- cleanup / postcondition:
  - Aspen source checkout is clean or the mutation exists only in a removed temporary copy

## Risks / Trade-offs

- Textual false positives around comments or strings -> start with fixture coverage for comments/strings and move matching behind Rust source tokenization if needed.
- Over-constraining existing wrappers -> require mechanism-level approved wrapper scopes and capacity-witness docs rather than banning wrappers categorically.
- Downstream proof noise from unrelated Aspen checks -> run only the focused consumer check and record unrelated full-flake failures as out of scope.
