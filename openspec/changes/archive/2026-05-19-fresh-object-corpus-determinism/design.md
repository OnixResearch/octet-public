## Context

The internal object corpus gate compares checked-in receipt bytes against fresh generation in the same checkout. Existing strong local determinism tests cover object emit/bundle behavior with scrubbed environments and temp target dirs. This change applies the same bounded local-repeatability pattern directly to object corpus receipts.

## Goals / Non-Goals

**Goals:**
- Exercise `cargo octet object corpus receipt` repeatedly with fresh `CARGO_TARGET_DIR` values.
- Assert byte-for-byte stability and explicit field stability for the compared replay fields.
- Preserve boundary language: this is local fresh-target repeatability evidence, not cross-machine determinism or correctness.

**Non-Goals:**
- No cross-machine/toolchain proof.
- No HIR/MIR semantic equivalence proof.
- No new receipt schema/version.

## Decisions

### 1. Test-level rail, not new CLI surface

**Choice:** Add integration-test helpers that execute the existing receipt command with scrubbed env and distinct temp target dirs.
**Rationale:** The behavior belongs to existing receipt generation. A new command would duplicate existing receipt/check semantics.
**Alternative:** Add a dedicated CLI `determinism` command now. Rejected as premature until the receipt/check surfaces settle further.

### 2. Assert bytes and named fields

**Choice:** Compare full receipt bytes and separately assert the stable fields that review cares about.
**Rationale:** Byte equality catches accidental nondeterministic serialization; field assertions make the proof intelligible in evidence.
**Alternative:** Compare only object-set hash. Rejected because schema/boundary/caveat drift would be missed.

## Risks / Trade-offs

**False overclaiming** → Evidence text and spec scenarios call this bounded local fresh-target repeatability only.

**Environment brittleness** → The helper clears the environment and restores only stable locale/timezone plus the fresh target dir.
