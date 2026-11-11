## Context

The archived rustc HIR spike and second-fixture changes established a closed set of selected fixture shapes: `checked-arithmetic` and `branch-control-flow`. The supplement emit/check path already uses this closed registry, and contract metadata exposes `selected_fixture_shapes`, but there is no direct command that shows operators which shapes are valid and what each shape means.

## Goals

- Make valid rustc HIR fixture shapes discoverable from the CLI.
- Keep the output machine-readable for evidence capture and human-readable for operator use.
- Derive inventory entries from the closed fixture registry so emit/check and inventory cannot silently diverge.
- Preserve selected-fixture/toolchain-scoped non-proof boundaries.

## Non-Goals

- Accepting arbitrary source paths or function names.
- Expanding beyond the two checked-in fixture shapes.
- Reclassifying HIR support as general or semantic.

## Decisions

### 1. Add a fixture inventory subcommand

**Choice:** Add an `object supplement fixtures` read-only command scoped by `--family hir --spike rustc-hir-fixture` with `--output-format human|json`.

**Rationale:** It keeps discovery near the existing supplement emit/check commands and makes fixture inventory evidence easy to capture.

**Rejected:** Encoding this only in `--help`; help text is useful, but it is harder to machine-check for exact fixture metadata and boundaries.

### 2. Reuse the closed fixture registry

**Choice:** Generate inventory entries from the same closed registry used by rustc HIR supplement emit/check.

**Rationale:** A separate hand-written list would invite drift between accepted selectors and documented selectors.

### 3. Emit boundary metadata with inventory entries

**Choice:** Include default marker, source path, function name, fixture shape, support status, and explicit non-proof fields in JSON, with concise equivalent human output.

**Rationale:** Inventory output should be enough for evidence review without implying arbitrary HIR support.

## Risks / Trade-offs

- **CLI surface churn:** Keep the command read-only and under `object supplement` to limit scope.
- **Overclaim wording:** Tests/evidence must assert selected-fixture-backed/toolchain-scoped wording and false non-proof flags.
- **Registry drift:** Mitigate by deriving entries from the registry and adding tests comparing inventory to accepted selectors.
