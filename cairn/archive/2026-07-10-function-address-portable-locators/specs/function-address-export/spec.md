# Function Address Export Specification

## Purpose

Make function-address records and baselines portable across checkout roots.

## Requirements

### Requirement: Function-address paths are workspace-relative
r[octet.function_address_export.portable_locators.relative_paths] Octet MUST emit function-address source-envelope and record paths as slash-normalized members relative to the Cargo workspace root.

#### Scenario: Relocated checkout has stable paths
r[octet.function_address_export.portable_locators.positive]
- GIVEN the same Rust source tree is checked in two different filesystem roots
- WHEN function-address export runs in both roots
- THEN emitted source paths and locators MUST match without embedding either absolute root.

### Requirement: Locators exclude host-specific prefixes
r[octet.function_address_export.portable_locators.locator] Octet MUST derive function locators from normalized relative source path and function display name, not from absolute host paths.

#### Scenario: Duplicate normalized locator fails closed
r[octet.function_address_export.portable_locators.negative]
- GIVEN two function records normalize to the same relative path and function name
- WHEN Octet builds the function-address report
- THEN Octet MUST report a duplicate-locator diagnostic before downstream receipts consume the report.

### Requirement: Absolute baseline compatibility is bounded
r[octet.function_address_export.portable_locators.baseline_compat] Octet MAY accept legacy absolute-path baselines only when each path resolves under the current workspace root and can be normalized to the new relative form.

#### Scenario: Escaping baseline is rejected
r[octet.function_address_export.portable_locators.validation]
- GIVEN a baseline entry contains an absolute path outside the workspace or a `..` escape
- WHEN function-address checking loads the baseline
- THEN Octet MUST reject the entry with a deterministic path-boundary diagnostic.
