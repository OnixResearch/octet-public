## Context

The README names Octet as the canonical product and says Rust source attrs and domain groups use `octet::...`. The lint declarations and group constants currently expose `tigerstyle::...` in several implementation and fixture paths. Prior notes also warn that naïve tool-lint migration can break source-attribute activation and no-todo/no-panic escape-hatch contracts.

## Decisions

### 1. Make one namespace decision explicit

**Choice:** For the current release line, keep `tigerstyle::` as the canonical source-attribute namespace and emitted group namespace while retaining Octet command/config product names.

**Rationale:** Live lint declarations, snapshots, UI fixtures, and runner metadata already agree on `tigerstyle::`. Documenting that behavior avoids aspirational `octet::` guidance until a safe tool-lint migration can preserve custom escape hatches.

### 2. Preserve compatibility with tests

**Choice:** Keep bare lint-name attrs and legacy config sections working for a defined compatibility window unless a future migration explicitly removes them.

**Rationale:** Existing downstream consumers use legacy attrs. Breaking them requires evidence, migration docs, and UI fixtures.

### 3. Migrate no-todo/no-panic carefully

**Choice:** Treat lints with custom escape-hatch semantics as special cases during namespace migration.

**Rationale:** Tool-scoped rustc allows can suppress lints before custom justification logic runs. The migration must not silently weaken escape-hatch enforcement.

### 4. Align runner output with docs

**Choice:** Emitted group names in JSON/SARIF and docs should match the selected canonical namespace, with compatibility aliases described separately.

**Rationale:** Release evidence should present one product identity and avoid surprising consumers with old names.

## Risks / Trade-offs

- Full migration can trigger UI snapshot churn. Mitigate by batching fixtures and preserving compatibility tests.
- Canonical `octet::...` attrs may require careful `declare_tool_lint!` conversion. Mitigate by moving one family at a time and validating source attrs in standalone fixtures.
- Deferring migration while docs say `octet::...` creates user confusion. Mitigate by updating docs immediately if deferral is chosen.

## Validation

- Positive: canonical namespace attrs and groups work in standalone UI fixtures.
- Negative: unsupported namespace spellings fail with clear rustc or Octet diagnostics.
- Compatibility: supported legacy namespace attrs and bare config keys continue to work until the documented removal boundary.
