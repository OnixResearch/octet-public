## Context

Octet already covers many Clippy-style policies under Tiger/Octet names. The remaining useful delta from `indexable-inc/clippy` is a small set of fork-specific restriction lints that are policy-oriented and fit Octet's mission.

The new lints span three existing domains:

- safety: `drop_must_use`, `fallible_int_fallback`
- structure: `magic_number`, `excessive_file_length`, `module_file_count`
- naming/imports: `underscore_in_module_filename`, `non_trait_imports`, `renamed_imports`, `path_segment_repetition`

## Goals / Non-Goals

**Goals:**

- Add first-class Octet lint requirements for the nine Indexable-inspired policies.
- Keep lint names canonical, config-validatable, and registered in normal Octet groups/metadata.
- Add focused UI fixtures and saved evidence for every scenario.
- Prefer conservative detection over broad text matching where type/source-map information is ambiguous.

**Non-Goals:**

- Vendor or copy the `indexable-inc/clippy` codebase wholesale.
- Replace upstream Clippy or duplicate every upstream Clippy lint.
- Rename existing Octet lints to Clippy names.
- Make all lints deny-by-default.

## Decisions

### 1. Implement as native Octet lints

**Choice:** Implement each policy as a native Octet lint under the existing `safety`, `structure`, or `naming` modules.

**Rationale:** Native lints preserve Octet's diagnostics metadata, config handling, group registration, UI harness, and saved evidence shape.

**Alternative:** Depend on `indexable-inc/clippy` directly. Rejected because it would couple Octet's public surface to a forked Clippy distribution and bypass Octet's receipt/metadata conventions.

### 2. Use conservative data sources per lint class

**Choice:**

- Use type information for `drop_must_use` and `fallible_int_fallback`.
- Use source-map/file-system path information for `excessive_file_length`, `module_file_count`, and `underscore_in_module_filename`.
- Use AST/HIR import and item-path information for import/name lints.

**Rationale:** These policies are prone to false positives if implemented as raw source text scans.

**Alternative:** Add early string-scanning passes. Rejected except where source-map paths are the actual lint target.

### 3. Keep explicit defaults and thresholds configurable

**Choice:** Add config fields for at least file length, module file count, and future magic-number allowlists/thresholds where practical.

**Rationale:** Structure thresholds are project policy knobs, while safety lints should remain semantic and warn-by-default.

**Alternative:** Hardcode all thresholds. Rejected because `excessive_file_length` and `module_file_count` are inherently project-scale policy choices.

### 4. Treat compatibility aliases as implementation detail

**Choice:** The canonical new lint names are the Indexable-inspired snake-case names, with bare compatibility group aliases following Octet's existing one-lint alias pattern.

**Rationale:** Users should be able to configure and allow/deny these lints the same way as existing Octet lints.

## Risks / Trade-offs

**Path-sensitive false positives** → Keep generated/test/fixture path exclusions explicit and verify with fixture directories.

**Import-policy restrictiveness** → Allow trait imports, `as _`, and documented public compatibility re-exports so normal Rust method-resolution and API patterns remain usable.

**Magic-number noise** → Exempt conventional local literals and require tests for named constants and unit-bearing names.

**Must-use detection drift across rustc versions** → Prefer rustc semantic metadata where available and isolate helper logic with focused unit/UI tests.

## Validation Plan

- Add one focused UI fixture per scenario group.
- Add config/registration tests proving every new lint appears in inventory, metadata, groups, and JSON output metadata.
- Save deterministic transcripts under `openspec/changes/archive/2026-05-18-add-indexable-policy-lints/evidence/`.
- Run `openspec validate add-indexable-policy-lints --strict` before implementation and before archive.
