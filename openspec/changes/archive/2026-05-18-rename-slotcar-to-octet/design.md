## Context

The previous rename to `slotcar` made the lint runner's constrained-motion identity coherent, but the project direction now points toward a Rust fork / language distribution. The name needs to scale from lint runner to language/toolchain identity without losing the core idea: fast systems code under explicit rails. `Octet` carries that forward through a chemistry metaphor: stable shells are inert by default, and controlled interaction requires complete, explicit interfaces.

Current repo state still has several historically valid identity layers:

- `slotcar` as current product/tool identity
- `tigerstyle` as compatibility command/env/namespace and source philosophy
- `Tiger Style` as the coding-principle source language
- policy IDs and archived evidence that may intentionally retain older names

The rename must therefore be staged and classified, not a blind text replacement.

## Goals / Non-Goals

**Goals:**

- Establish `Octet` as the canonical project/toolchain identity.
- Specify the complete public naming map before implementation.
- Keep compatibility aliases explicit and testable.
- Require deterministic evidence for remaining legacy names.
- Preserve Tiger Style philosophy references where they are semantically correct.

**Non-Goals:**

- Change lint behavior or standards policy semantics.
- Remove compatibility aliases in this same change.
- Rewrite archived evidence/history wholesale.
- Publish a release or change upstream remotes.

## Decisions

### 1. Use Octet as the top-level identity

**Choice:** Rename the public project/toolchain identity to `Octet` / `octet`.

**Rationale:** Octet evokes the stable full-shell state behind noble-gas inertness. It maps directly to Rust composition boundaries: APIs, effects, capabilities, unsafe seams, proof rails, and review identity should be closed and non-reactive unless an explicit interface permits interaction. It keeps the Slotcar promise of constrained motion, but at language/distribution scale.

**Alternative:** Keep `slotcar` for the runner and introduce Octet only later as a fork name. Rejected because dual current identities would make docs, commands, and generated receipts drift.

**Implementation:** Add a naming map to README/docs, then migrate manifests, CLI help, Nix outputs, templates, tests, and policy artifacts around that map.

### 2. Canonical command is `cargo octet`

**Choice:** `cargo-octet` / `cargo octet` becomes canonical. `cargo slotcar` and `cargo tigerstyle` are allowed only as documented compatibility aliases.

**Rationale:** Consumers need one preferred command. The older names still protect current workflows during migration, but new docs and generated outputs should converge on Octet.

**Alternative:** Keep `cargo tigerstyle` as command while renaming only the repo. Rejected because command/output identity is the primary consumer surface.

**Implementation:** Rename the runner package/binary or wrap it so the canonical binary is `cargo-octet`; update help text, SARIF driver metadata, sidecars, docs, and integration tests. Compatibility aliases must call the same engine path.

### 3. Canonical config is `[octet]`

**Choice:** New consumer configuration uses `[octet]`, `[octet.lint_levels]`, and `[workspace.metadata.octet]`.

**Rationale:** Config is a durable public API. Keeping `[slotcar]` or `[tigerstyle]` as canonical would undermine the rename and complicate future fork/toolchain docs.

**Alternative:** Preserve old config names indefinitely as equal peers. Rejected because equal peers make precedence ambiguous and increase docs burden.

**Implementation:** Add Octet-first parsing with explicit precedence over legacy sections. Keep legacy compatibility only as documented migration behavior.

### 4. Classify remaining legacy names

**Choice:** Completion requires a deterministic name-classification artifact for non-archived legacy terms.

**Rationale:** This repo has legitimate `Tiger Style` philosophy references and archived evidence. A raw grep must not be treated as failure, but every retained occurrence should have a reason.

**Alternative:** Accept manual review only. Rejected because rename correctness is otherwise too easy to overclaim.

**Implementation:** Add or adapt a helper that reports occurrences and categories, with zero `needs-fix` rows required.

## Risks / Trade-offs

**Large mechanical churn** → Keep the OpenSpec tasks split by surface and run narrow verification after each phase.

**Compatibility ambiguity** → Document aliases as transition-only and assert canonical output names Octet.

**Generated artifact drift** → Regenerate Nickel-derived JSON/fixtures after `.ncl` edits and run the maintained policy validation/export checks.

**Over-renaming Tiger Style philosophy** → Require classification categories and preserve philosophy wording where it describes the source coding style rather than the tool identity.
