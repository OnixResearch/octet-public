## Context

The repo currently uses `tigerstyle` in Cargo package names, Dylint/library names, `cargo-tigerstyle`, Nix outputs, docs, examples, tests, templates, diagnostics, receipts, and OpenSpec configuration. Some occurrences describe the source coding philosophy and should remain; others are product/tool identity and should become `slotcar`.

## Goals / Non-Goals

**Goals:**
- Make `slotcar` the canonical repo/tool name across install, invocation, docs, Nix, and emitted tool metadata.
- Keep any transitional `tigerstyle` alias deliberate, documented, and covered by tests.
- Preserve legitimate Tiger Style philosophy and compatibility/namespace text.

**Non-Goals:**
- Change lint behavior, diagnostic triggering, or severity semantics.
- Remove compatibility aliases unless implementation proves it is safe and records a separate decision.
- Rewrite archived evidence beyond classification.

## Decisions

### 1. Rename product surfaces first, classify leftovers second

**Choice:** Update canonical manifests, binary names, Nix outputs, CLI help, docs, templates, tests, and emitted tool metadata to `slotcar`, then run a deterministic classification over remaining non-archived `tigerstyle` strings.

**Covers:** Scenario: Canonical install and usage docs say slotcar; Scenario: Philosophy references remain distinguishable; Scenario: Structured outputs name slotcar

**Rationale:** A repo-wide blind replacement would corrupt legitimate Tiger Style philosophy references and archived evidence. A classification artifact makes intentional leftovers reviewable.

**Alternative considered:** Replace every `tigerstyle` string with `slotcar`. Rejected because lint namespaces, compatibility aliases, archived transcripts, and philosophy text need different handling.

**Implementation:** Rename Cargo packages/directories where feasible, Nix app/package attributes, CLI usage/help strings, docs, templates, generated-output expected strings, and receipt/tool metadata. Add or update a helper that emits `name-classification.txt` with category and location for every remaining non-archived occurrence.

**Boundary cases:** Archived `openspec/changes/archive/**` evidence may keep historical strings. Tool lint namespaces may remain temporarily if rustc compatibility requires it, but must be classified as `namespace` and not described as the product name.

### 2. Canonical CLI with compatibility alias

**Choice:** Make `cargo slotcar` / `cargo-slotcar` canonical and keep `cargo tigerstyle` only if it delegates through the same code path and is documented as a compatibility alias.

**Covers:** Scenario: Cargo subcommand uses slotcar; Scenario: Compatibility alias is explicit

**Rationale:** Consumers need a stable migration path, but new docs and outputs should converge on one canonical name.

**Alternative considered:** Break `cargo tigerstyle` immediately. Rejected because existing downstream consumers may rely on the current command and templates.

**Implementation:** Rename the wrapper package/binary to `cargo-slotcar`; if compatibility is kept, provide an alias wrapper or installable compatibility binary that calls the canonical implementation. Tests compare minimal fixture behavior for canonical and compatibility invocations.

**Boundary cases:** Compatibility output may mention the old invocation only in deprecation/compatibility text; machine-readable tool identity remains `slotcar` unless a field is explicitly a historical command alias.

## Deterministic Proof Plan

- scenario or requirement:
  - Scenario: Canonical install and usage docs say slotcar
  - Scenario: Philosophy references remain distinguishable
- artifact path:
  - `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt`
- command / helper:
  - `nix develop -c bash -c '<repo rename classification helper>'`
- exact expected output / acceptance signal:
  - artifact contains every remaining non-archived `tigerstyle` location categorized as `philosophy`, `compatibility`, `namespace`, or `archived`, and contains zero `needs-fix` rows
- ordering / prerequisites:
  - run after manifests/docs/templates/tests have been renamed
- cleanup / postcondition:
  - no temporary classification files outside the evidence path

- scenario or requirement:
  - Scenario: Cargo subcommand uses slotcar
  - Scenario: Structured outputs name slotcar
  - Scenario: Compatibility alias is explicit
- artifact path:
  - `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`
- command / helper:
  - `nix develop -c bash -c '<focused cargo slotcar integration check>'`
- exact expected output / acceptance signal:
  - artifact shows `cargo slotcar check` succeeds on a minimal fixture, output tool identity is `slotcar`, and any retained `cargo tigerstyle` alias succeeds with equivalent findings
- ordering / prerequisites:
  - run after canonical wrapper and any alias are implemented
- cleanup / postcondition:
  - temporary consumer fixture and target dirs are removed or isolated under test temp dirs

## Risks / Trade-offs

- Broad rename churn can invalidate snapshots and evidence -> mitigate with focused fixture updates and saved classification/CLI transcripts.
- Rust/Dylint tool-lint namespace migration may be entangled with source attributes -> keep namespace leftovers explicit and do not force namespace removal in this rename.
- Downstream consumers may still invoke `cargo tigerstyle` -> keep a compatibility alias unless tests and docs justify removing it later.
