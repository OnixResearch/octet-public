## Context

Octet's object supplement contract already includes HIR, THIR, and MIR families, but the runtime CLI only parses HIR/MIR. THIR is therefore documented as unsupported-planned yet not directly exercisable through the emit/check replay seam.

## Goals / Non-Goals

**Goals:**
- Add `thir` to the supplement family parser for pilot-scope emit/check.
- Emit deterministic THIR supplement records that are explicitly `unsupported-planned`.
- Preserve all evidence-only/non-proof boundaries and replay comparison behavior.

**Non-Goals:**
- Implement rustc THIR extraction.
- Add THIR fixture inventory or selector support.
- Change HIR/MIR semantics or aggregate Valence gates beyond focused tests.

## Decisions

### 1. Model THIR as a runtime family with unsupported-planned status

**Choice:** Extend `SupplementFamilyArg` and `ObjectSupplementFamily` with `Thir`, then generate THIR records through a dedicated `pilot_thir_supplement` helper.

**Rationale:** This keeps the CLI, runtime receipts, and contract metadata aligned while preventing accidental extractor claims.

**Alternative:** Continue rejecting `--family thir`. Rejected because operators cannot replay the documented unsupported-planned THIR surface.

**Implementation:** `pilot_thir_supplement` uses the same Function Object v1 identity input as HIR/MIR but records only unsupported-planned THIR observations and caveats. The emitted supplement fields are exact and reviewable: `schema = "octet.function-object-thir-supplement.v1"`, `schema_version = 1`, `family = "thir"`, `status = "unsupported-planned"`, `function_object_hash`, `source_path`, `function_name`, `rustc_version`, `target_triple`, `extractor.name = "octet-thir-pilot-unsupported-planned"`, `extractor.version`, `observation_hash`, `observations`, `extraction_caveats`, and `boundary`. The `observations` map must pin these entries: `thir_context.status = "unsupported-planned"` with detail `THIR extraction is not implemented; this record preserves a fail-closed runtime family seam`; `typed_hir_lowering.status = "unsupported-planned"` with detail `typed HIR/THIR lowering observations require future extractor evidence before support`; and `support_boundary.status = "unsupported-planned"` with detail containing the selected object hash plus `unsupported-planned evidence only; no correctness, semantic-equivalence, pure-cache, or cross-toolchain claim`. The caveat list must include `family = "thir"`, `status = "unsupported-planned"`, and detail `Octet has not implemented a THIR extractor; matching fail-closed receipts do not imply THIR support`. Boundary fields must keep `claims_correctness = false`, `claims_semantic_equivalence = false`, `claims_pure_cache_eligibility = false`, and `claims_cross_toolchain_stability = false`. The focused `thir_supplement_emit_and_check_remain_fail_closed` test snapshots these names/statuses/details enough to block accidental support overclaims.

### 2. Keep fixture inventory HIR-only

**Choice:** Do not add THIR to `object supplement fixtures`.

**Rationale:** The existing inventory is a closed list of checked-in rustc HIR fixture shapes. THIR has no deterministic fixture extractor yet.

**Alternative:** Return an empty THIR inventory. Rejected for this slice because it would change the fixture-inventory contract instead of closing the runtime family gap.

### 3. Reuse supplement check comparison semantics

**Choice:** THIR check compares the same receipt-critical fields as other supplement families: schema, schema_version, family, status, function_object_hash, source_path, function_name, rustc_version, target_triple, extractor name/version, observation_hash, boundary, and observation/caveat-derived hash inputs through the existing comparison report.

**Rationale:** This keeps replay deterministic and fail-closed without introducing a second checker contract.

**Alternative:** Treat unsupported-planned THIR receipts as always passing. Rejected because that would make the runtime seam non-reviewable.

**Implementation:** `object supplement check --family thir` must report `status = "pass"` only when every comparison reports `status = "match"`; mismatches keep the existing non-zero exit/fail report behavior and preserve boundary fields.

## Risks / Trade-offs

**Overclaiming THIR support** → Mitigated by `unsupported-planned` status, unsupported observations/caveats, focused assertions, and unchanged non-proof boundaries.

**Verification gaps** → Mitigated by exact commands: `cargo test -p cargo-octet thir_supplement -- --nocapture` proves THIR schema/status/boundary emit and replay, `cargo test -p cargo-octet supplement -- --nocapture` proves parser/docs/runtime supplement regressions without widening fixture inventory, `openspec validate add-thir-supplement-fail-closed --strict --json` validates the change, and final `openspec validate --all --strict --json` plus `git diff --check` close the archive.
