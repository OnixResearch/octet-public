## Context

The current corpus checker compares critical receipt fields after parsing a saved receipt and regenerating the current object corpus. Parse failures, missing source inputs, unsupported schema variants, and empty corpus selections can still surface as generic errors. For CI and future Cairn replay use, these must be deterministic report records.

## Goals / Non-Goals

**Goals:** typed failure kinds, deterministic JSON/human reports, explicit fail-closed schema/version behavior, and negative tests for invalid receipts and invalid selections.

**Non-Goals:** JSON Schema publication, receipt migration tooling, cross-version compatibility, correctness proofs, semantic-equivalence proofs, or HIR/MIR-based validation.

## Decisions

### 1. Fail closed before corpus comparison

**Choice:** Validate receipt shape and source selection before comparing regenerated evidence fields.

**Rationale:** Missing fields or unsupported schema versions should not be confused with source drift.

**Alternative:** Let every missing field become a `null` comparison mismatch. Rejected because operators need typed diagnostics for schema errors.

### 2. Report invalid checks through the same report envelope

**Choice:** Extend `ObjectCorpusCheckReport` with an `errors` array while preserving `comparisons` for valid-but-drifted receipts.

**Rationale:** Consumers can parse one stable envelope for match, mismatch, and invalid-input outcomes.

**Alternative:** Return only stderr for malformed inputs. Rejected because CI/replay consumers need machine-readable failure data.

### 3. Keep semantics boundary unchanged

**Choice:** The hardened checker still validates evidence identity fields only.

**Rationale:** Schema hardening must not imply correctness, semantic equivalence, or cross-machine determinism.

## Risks / Trade-offs

**Report schema churn** → Use additive `errors` field and cover JSON output with tests.

**Over-strict selection validation** → Restrict only duplicate, missing, and non-Rust explicit paths in this slice; directory/glob expansion remains the caller shell/tool's job.
