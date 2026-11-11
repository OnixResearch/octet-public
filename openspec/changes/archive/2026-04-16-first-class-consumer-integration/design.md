## Context

Tigerstyle's current downstream rollout path grew around a copied shell script.
That script now owns too much policy: toolchain setup, pinned `dylint_driver`
bootstrap, lockfile hygiene, default scope, and no-argument behavior. Aspen
proved the workflow is useful, but Seaglass showed the core problem clearly:
consumer integration behavior lived in a copy instead of a product surface.

This change intentionally does not preserve backward compatibility for the
script-copy workflow. The supported path will become a first-class Rust engine,
workspace metadata, and Nix outputs.

## Goals / Non-Goals

**Goals:**
- Provide one canonical downstream integration engine instead of copied shell
  logic
- Move consumer runner policy into workspace metadata
- Expose a first-class Nix app / check surface that uses the same engine
- Make local helper commands policy-thin wrappers over the same engine
- Keep consumer `Cargo.lock` read-only during check runs

**Non-Goals:**
- Replacing consumer-owned `dylint.toml` lint tuning
- Preserving the copied shell-wrapper path as a supported integration path
- Designing a broad project generator beyond what is needed to wire the new
  engine into consumers

## Decisions

### 1. Canonical implementation is a shared Rust engine plus `cargo-tigerstyle`

**Choice:** Introduce a new `cargo-tigerstyle` workspace member that owns the
consumer-integration engine plus the `cargo-tigerstyle` binary. The engine is
implemented as internal Rust modules in that crate, and every supported entry
point — direct `cargo tigerstyle check`, `nix run`, and first-party Nix check
helpers — invokes that same binary rather than a second public engine crate or
copied shell path. The binary is the canonical published UX, and it wraps
`cargo dylint` internally rather than replacing Dylint with a second lint
runner.

**Rationale:** The shell template already proved the behavior we need, but bash
is the wrong place for a long-lived product surface. A Rust engine inside one
workspace member is easier to unit test, easier to ship through Cargo and Nix,
and removes copy drift across consumers while preserving the existing
Dylint-based lint execution model. Keeping the engine inside `cargo-tigerstyle`
also avoids disturbing the existing `tigerstyle` lint crate layout.

**Alternative considered:** Keep improving the bash template. Rejected because
it still duplicates policy into every consumer and leaves behavior tied to a
copy operation.

### 2. Consumer runner policy lives only in `[workspace.metadata.tigerstyle]`

**Choice:** Use `[workspace.metadata.tigerstyle]` in the consumer workspace
`Cargo.toml` as the sole supported place for runner policy. The initial schema
contains two required array-of-string keys:

```toml
[workspace.metadata.tigerstyle]
default_scope = ["-p", "aspen-time", "-p", "aspen-core"]
cargo_check_args = ["--lib"]
```

Missing either key is a configuration error. `default_scope` defines the
no-argument package / workspace selection, and `cargo_check_args` defines the
no-argument `cargo check` tail arguments passed after `--`. The engine treats
those arrays as raw token lists owned by the consumer workspace: when the caller
passes no explicit `-- ...` tail, the engine forwards `cargo_check_args`
unchanged; when the caller does pass an explicit tail, that CLI tail replaces
metadata defaults instead of merging with them.

**Rationale:** Workspace metadata keeps runner policy close to the workspace it
controls, fits Cargo's existing configuration model, and avoids a second file
format just for command defaults. Replace-not-merge semantics keep the contract
simple and match how explicit package-scope overrides already beat metadata
defaults.

**Alternative considered:** `tigerstyle.toml`. Rejected because it would add
another top-level config file while `dylint.toml` already remains in place for
lint tuning.

### 3. Nix uses the same Rust engine, not a second shell path

**Choice:** Expose a flake app for `nix run` and a first-party helper for
consumer `checks`; both call the same Rust engine and feed it the same consumer
metadata contract.

**Rationale:** Nix consumers need reproducible integration, but they should not
get a separate implementation. One engine keeps local runs, `nix run`, and
`nix flake check` behavior aligned.

**Alternative considered:** Keep `mkConsumerCheck` as a shell-heavy special
case. Rejected because that would recreate the same split-brain problem inside
Nix.

### 4. Driver bootstrap stays internal and fail-closed

**Choice:** Port the proven lockfile-safe behavior from the fixed template into
Rust: bootstrap any pinned `dylint_driver` in isolated temp state, treat the
consumer workspace as read-only, and fail after restoration if the check path
mutates `Cargo.lock`. The engine also owns nix-managed nightly resolution so
consumers do not need `rustup` wrapper scripts.

**Rationale:** This is the critical correctness property discovered during the
Seaglass run. It belongs in the engine, not in repo memory or post-run cleanup.

**Alternative considered:** `cargo --locked dylint`. Rejected because it still
allowed the copied wrapper path to leave `[[patch.unused]] dylint_driver` in the
consumer lockfile.

### 5. Exit codes distinguish lint findings from integration failures

**Choice:** `cargo tigerstyle check` returns exit code `0` on successful runs,
including warning-only lint findings; exit code `1` when `cargo dylint` reports
error-level lint findings; and exit code `2` for tigerstyle integration or
configuration failures such as invalid metadata, toolchain resolution failure,
driver bootstrap failure, or fail-closed lockfile restoration.

**Rationale:** CI callers need to distinguish "fix the code or lint level" from
"fix the tigerstyle setup." A two-failure-code contract keeps the UX stable
without inventing a large exit-code taxonomy.

**Alternative considered:** Pass through every subprocess exit code verbatim.
Rejected because config and bootstrap failures would be harder for consumers and
saved proof to classify consistently.

### 6. Repo-local helpers are delegation-only

**Choice:** Keep any generated or documented repo-local entry points
(`just tigerstyle`, `cargo xtask tigerstyle`, CI snippets) as thin delegations
that pass through to the canonical engine.

**Rationale:** Developers still want short commands, but helper convenience must
not become a second policy surface.

### 7. Non-Nix installs use an embedded source bundle, not runtime path discovery

**Choice:** Package `cargo-tigerstyle` with an embedded deterministic source
bundle of the tigerstyle lint workspace plus a BLAKE3 content hash. At runtime,
the engine extracts that bundle under
`${XDG_CACHE_HOME:-~/.cache}/tigerstyle/sources/<blake3>/`, verifies the cached
contents by the same hash marker, and uses atomic temp-dir-then-rename
extraction so partial or corrupted cache entries get replaced cleanly on the
next run. Nix packages may instead point directly at a deterministic store path,
but they use the same source contents and the same engine behavior once the
source root is resolved.

**Rationale:** This removes `TIGERSTYLE_PATH`, avoids network fetches at check
time, works for `cargo install`, and gives the CLI one deterministic source to
run everywhere while keeping cache reuse explicit and reviewable.

**Alternative considered:** Fetch the tigerstyle repo or lint crate from git at
runtime. Rejected because it would add network dependence, version drift, and a
second resolution policy outside the consumer workspace.

## Verification Strategy

The implementation must replace the current template-only proof with
first-class engine evidence:

- Rust unit and integration tests centered on `cargo-tigerstyle`, including
  metadata parsing, explicit scope overrides, `cargo_check_args`
  replace-not-merge behavior, exit-code classification, lockfile-preserving
  success, and fail-closed lockfile restoration
- disposable consumer-workspace fixtures exercised from Rust integration tests,
  with checked-in representative manifests and thin-wrapper examples under
  `tests/fixtures/consumer-integration/`
- a regression test covering nix-managed nightly toolchain resolution without a
  consumer wrapper script
- Nix verification inside `nix flake check`, not only `nix develop`, proving
  the flake app and flake check helper call the same Rust engine and honor the
  same metadata contract
- downstream proof artifacts from a disposable consumer workspace showing the
  CLI path and the Nix path run without copied tigerstyle scripts

The existing wrapper-specific integration proof is transitional only;
equivalent coverage must move to the new CLI / engine tests before the old
wrapper surface is removed.

## Risks / Trade-offs

- **[Source discovery for the lint crate]** → The CLI still needs a reliable way
  to find the tigerstyle lint source or packaged artifact. Mitigation: make the
  engine own source resolution with explicit overrides for development and a
  deterministic packaged path for Nix.
- **[Breaking existing copied-script consumers]** → Consumers using the old
  template will need to migrate. Mitigation: document a single migration path,
  update known downstream consumers, and remove active docs that point to the
  old script.
- **[CLI + Nix coupling drift]** → Separate implementations could diverge.
  Mitigation: Nix outputs invoke the same `cargo-tigerstyle` binary, and
  regression tests cover both metadata parsing and lockfile invariants under
  `nix flake check`.

## Migration Plan

1. Add the `cargo-tigerstyle` workspace member and internal engine modules.
2. Define and validate the `[workspace.metadata.tigerstyle]` schema, including
   explicit CLI replacement semantics for `cargo_check_args`.
3. Port toolchain resolution, driver bootstrap, lockfile guard, and exit-code
   classification from the fixed template into Rust.
4. Wire flake apps / check helpers to the same binary and verify them through
   `nix flake check`.
5. Add disposable-consumer fixtures and proof artifacts for CLI and Nix parity.
6. Update downstream rollout docs and examples to use the new engine.
7. Remove the copied shell-wrapper file as a supported active integration
   surface once the engine path is ready.

## Open Questions

- None at design time. If the embedded source bundle or cache path proves too
  heavy in implementation, open a follow-up change rather than reviving the old
  script-copy contract.
