## ADDED Requirements

### Requirement: Standardized Diagnostic Structure

Every lint diagnostic MUST include:

1. A one-line warning message describing the violation
2. A help note in the exact form `Tiger Style (<principle>): <guidance>`;
   for lints with a concrete approved pattern, `<guidance>` MUST include an
   appended `Example: <approved pattern>` clause in the same help note
3. A note in the exact form `see Tiger Style guide: <section>`

#### Scenario: Developer sees lint for the first time

- GIVEN a developer runs tigerstyle and encounters `ambient_clock`
- WHEN they read the diagnostic
- THEN they see the principle name, the approved pattern, and where to
  read more without opening a separate document

#### Scenario: Diagnostic wording stays consistent across lints

- GIVEN two different tigerstyle lint diagnostics
- WHEN a developer compares their help and note lines
- THEN both diagnostics use the same `Tiger Style (...)` help prefix and
  the same `see Tiger Style guide: ...` note prefix

### Requirement: Canonical Tool-Scoped Tigerstyle Namespace

Every consumer-facing tigerstyle lint MUST be registered as a true tool lint in
canonical `tigerstyle::<lint_name>` form. Tool-scoped domain groups MUST be
registered against those true tool-lint IDs rather than ad-hoc plain-lint
aliases.

Standalone raw-rustc crates that use tool-scoped tigerstyle attrs MUST compile
on the supported nightly toolchain and opt into `#![feature(register_tool)]`
plus `#![register_tool(tigerstyle)]`.

#### Scenario: Tool-scoped per-lint attr resolves against a true tool lint

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN it uses a source-level attr like `#[allow(tigerstyle::no_unwrap)]`
- THEN rustc resolves that attr against the real `tigerstyle::no_unwrap` lint
  instead of a plain-lint compatibility shim

### Requirement: Tool-Lint Migration Precondition

Before converting all 25 consumer-facing lints, the project MUST validate the
planned migration mechanism on a single representative lint under the repo's
Dylint harness.

That precondition MUST prove both of these points together:

- `declare_tool_lint!(pub tigerstyle::...)` works for the representative lint
  under the project's Dylint loading path
- the intended bare compatibility mechanism — a one-member bare alias
  registered through `lint_store.register_group(...)` — still resolves the
  current bare config / lint-level path for that representative lint

#### Scenario: Representative spike proves canonical and compatibility paths

- GIVEN representative lint `no_unwrap`
- WHEN the project migrates only that lint to
  `declare_tool_lint!(pub tigerstyle::no_unwrap)` and registers a one-member
  bare compatibility alias for it
- THEN canonical tool-scoped attrs resolve in the supported standalone UI path
- AND bare `disabled_lints`, bare `[tigerstyle.lint_levels]`, and the current
  bare lint-level path still resolve for `no_unwrap`
- AND the full 25-lint migration does not proceed until that spike passes

### Requirement: Lint Groups By Spec Domain

The project MUST register lint groups matching spec domains:

- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

Consumers MUST be able to use group-level `#[allow]` and `#[deny]`
attributes when compiling on the supported nightly toolchain and opting into
`#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]`.

#### Scenario: Suppressing a domain during phased rollout

- GIVEN a consumer in starter profile compiling on the supported nightly
  toolchain with `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN they add `#![allow(tigerstyle::safety)]` to a crate root
- THEN all safety-domain lints are suppressed for that crate

#### Scenario: Allowing one domain does not suppress another

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- AND the crate would otherwise trigger both a safety lint and a naming lint
- WHEN it adds `#![allow(tigerstyle::safety)]` to the crate root
- THEN the safety lint is suppressed
- AND the naming lint still emits a diagnostic

#### Scenario: Escalating one domain during cleanup

- GIVEN a consumer wants stricter naming cleanup and is compiling on the
  supported nightly toolchain with `#![feature(register_tool)]` and
  `#![register_tool(tigerstyle)]`
- WHEN they add `#![deny(tigerstyle::naming)]` to a crate root
- THEN naming-domain diagnostics in that crate become hard errors

## MODIFIED Requirements

### Requirement: Per-Lint Configuration Support

The project MUST continue to support disabling individual lints and overriding
lint severity through the consumer-owned `dylint.toml` even after migrating the
canonical lint namespace to `tigerstyle::<lint_name>`.

The consumer-owned `[tigerstyle]` section in `dylint.toml` MUST keep these
behaviors:

- `disabled_lints` continues to use bare lint names like `no_unwrap`
- `[tigerstyle.lint_levels]` continues to use bare lint names like `no_unwrap`
- bare config names resolve to the matching canonical tool lint without
  requiring consumers to rename config keys to `tigerstyle::...`
- `disabled_lints` still prevents disabled lints from registering, so it beats
  level overrides and source-level attrs

#### Scenario: Bare config key still upgrades the canonical tool lint

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code after the tool-lint migration
- THEN the canonical `tigerstyle::no_unwrap` lint emits an error instead of a
  warning

#### Scenario: Bare disabled key still beats source-level attrs

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- AND source code requests a source-level lint attribute for
  `tigerstyle::no_unwrap`
- WHEN the lint suite runs on code that would otherwise violate `no_unwrap`
- THEN `no_unwrap` still emits no diagnostic

#### Scenario: Disabled bare key keeps the lint out of groups and aliases

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite registers canonical domain groups and bare compatibility
  aliases
- THEN `no_unwrap` is omitted from those registrations
- AND no canonical group or bare compatibility path re-enables that lint
