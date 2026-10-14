## Context

The current Cargo manifests declare several compatibility binaries by pointing each target at the same `src/main.rs`. Cargo accepts this, but it emits warnings during normal checks. The warnings are not product behavior, but they make clean check output noisy and undercut the build-hygiene rail that treats new owned warnings as regressions.

## Decisions

### 1. Use tiny alias wrappers or a shared library entry point

**Choice:** Move shared command execution into a reusable Rust entry point, then give each binary target its own tiny wrapper source file.

**Rationale:** Cargo no longer sees one file reused by multiple binary targets, while the wrappers still dispatch to the same implementation. This keeps aliases mechanical and reviewable.

### 2. Preserve command identity and compatibility behavior

**Choice:** Alias wrappers must preserve existing command-line parsing, version output, help text, exit codes, Nix package names, and compatibility binary availability.

**Rationale:** Consumers may call the legacy commands directly. Removing Cargo warnings must not become a command-behavior migration.

### 3. Make warning text a regression assertion

**Choice:** Add a focused check that captures `cargo check` output and rejects the duplicate-target warning string for owned alias packages.

**Rationale:** A behavior-preserving manifest cleanup can regress silently unless the warning class is asserted directly.

## Risks / Trade-offs

- Moving `main.rs` logic into a library entry point can churn module visibility. Mitigate by keeping the public surface small, for example one `run_cli_from_env` shell entry point per package.
- Alias wrappers may change `argv[0]`-dependent behavior if parsing assumes one spelling. Mitigate by testing each canonical and compatibility binary.
- Nix package wrappers may need path updates if binary locations move. Mitigate with release-smoke assertions for every installed binary.

## Validation

- Positive: each canonical and compatibility binary prints the expected version/help and routes to the same command engine.
- Negative: a captured workspace check output containing `found to be present in multiple build targets` fails the build-hygiene regression test.
- Regression: `nix develop -c cargo check`, package-targeted tests, and release smoke remain passing with no duplicate-target warning text.
