## Context

The current `Config` struct in `src/config.rs` has five threshold fields.
Lint registration in `src/lib.rs` unconditionally registers all 26 lints.
Consumers use source-level `#[allow]` or the rollout playbook's social
conventions to control which lints fire. Downstream rollout guidance uses
three named profiles: smoke, boundary-first, and inventory-only.

## Goals / Non-Goals

**Goals:** Let consumers enable/disable individual lints and set per-lint
severity through `dylint.toml`.

**Non-Goals:** Runtime lint selection via CLI flags. Dylint does not
support passing per-lint flags through cargo; config-file-based control
is the right approach.

## Decisions

### 1. `disabled_lints` list suppresses registration

**Choice:** Lints named in `disabled_lints` are not registered with the
lint store at all.

**Rationale:** Skipping registration entirely means no traversal cost for
disabled lints. Source-level `#[allow]` only suppresses diagnostics but
the lint pass still runs.

**Implementation:** In `register_lints`, check `cfg.disabled_lints`
before each `register_lints` + `register_*_pass` call pair. If a lint
appears in both `disabled_lints` and `lint_levels`, the disable wins,
the level override is ignored, and config load emits a warning.

**Alternative:** Register all lints and set their default level to `Allow`
— rejected because it still runs the pass and can produce confusing
"lint unknown" messages when mixed with source-level attributes.

### 2. `lint_levels` override default severity

**Choice:** A `[tigerstyle.lint_levels]` table maps lint names to
`"allow"`, `"warn"`, or `"deny"`.

**Rationale:** Consumers may want `compound_assertion` at `deny` level
(hard error) while keeping `ambient_clock` at `warn`. This is different
from disabling — the lint still runs but its findings have different
severity.

**Implementation:** Validate `lint_levels` strings at config-load time,
normalize accepted values into an internal enum, and ignore invalid
values after emitting a warning. Registration then consults the validated
override map and feeds the effective per-lint level into rustc's normal
lint-level machinery while keeping the original lint statics registered.

**Alternative:** Rely on `#![warn(...)]` / `#![deny(...)]` in source —
rejected because it requires source changes in every crate.

### 3. Defaulted config fields

**Choice:** New fields use `#[serde(default)]` and empty-collection
defaults.

**Rationale:** Consumers should be able to opt into per-lint controls
without filling every config field up front.

**Implementation:** `disabled_lints: Vec<String>` defaults to empty.
`lint_levels: HashMap<String, String>` defaults to empty.

### 4. Validation at load time

**Choice:** Unknown lint names in `disabled_lints` or `lint_levels`,
invalid `lint_levels` values, and `lint_levels` entries for disabled
lints produce a warning during config load, not a hard error.

**Rationale:** Consumers should get actionable feedback without losing
an entire lint run over a typo or unfinished rollout edit.

### 5. Shared lint-name inventory

**Choice:** Config validation and registration derive lint names from one
shared inventory.

**Rationale:** Per-lint config is only trustworthy if accepted config
names and registered lint names stay in sync as new lints are added.

**Implementation:** Keep a single canonical lint inventory in the crate
and use it for both config validation and registration wiring.

## Risks / Trade-offs

**[Config drift from source-level attrs]** → A lint disabled in
`dylint.toml` but `#[deny]`'d in source will silently win at the config
level (lint never registered, so the source attr is inert). Mitigation:
document that `dylint.toml` disabled_lints takes precedence over source
attributes.

**[Testing complexity]** → Need to test config parsing, lint-disable
behavior, and level overrides. Mitigation: integration tests with
different `dylint.toml` configs and expected diagnostic counts.

**[Downstream rollout drift]** → This change updates consumer-facing
rollout docs/templates that Aspen uses as the primary downstream proof.
Mitigation: when those docs/templates change, save a human Aspen replay
transcript under the change evidence directory using the canonical repo
template path (`templates/tigerstyle-check.sh`) and the pinned Aspen
revision recipe.
