## Context

The internal corpus receipt command records source-token object identity for selected Octet Rust source files. The check command can fail closed on drift and malformed inputs. A checked-in gate fixture turns that machinery into a repository-level review guard.

## Decisions

### 1. Store the fixture under `cargo-octet/tests/fixtures/object-corpus/`

**Choice:** Commit `internal-corpus-receipt.json` beside cargo-octet integration fixtures.

**Rationale:** The fixture is owned by the `cargo-octet` integration test that replays it, and it should move with the CLI crate rather than OpenSpec archives.

### 2. Gate with existing `object corpus check` JSON output

**Choice:** Add an integration test that invokes the built `cargo-octet` binary with the checked-in receipt and `INTERNAL_OBJECT_CORPUS`.

**Rationale:** This exercises the operator-facing command exactly as CI would, including schema validation and no-correctness/no-semantic-equivalence boundary comparison.

### 3. Explicit fixture-refresh oracle

**Choice:** Save evidence that compares a freshly emitted receipt against the checked-in fixture and prints PASS assertions.

**Rationale:** When source changes intentionally alter object identity, maintainers need deterministic proof that the updated fixture was regenerated from the intended bounded corpus.

## Risks / Trade-offs

- The gate will fail when selected source files intentionally change. That is acceptable; the maintainer must refresh the receipt fixture in the same review.
- The fixture is bounded to local Octet source identity. It is not a cross-machine determinism proof or semantic correctness proof.
