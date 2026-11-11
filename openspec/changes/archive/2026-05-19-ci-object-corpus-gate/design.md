## Context

The existing `cargo test -p cargo-octet` path already includes the object corpus tests, but CI and operators benefit from a named flake check that can be invoked independently and referenced by docs/evidence.

## Goals / Non-Goals

**Goals:**
- Add a named Nix check for the internal object corpus gate.
- Cover fixture replay, byte-for-byte fixture freshness, contract metadata compatibility, and fresh-target receipt determinism through the existing integration tests.
- Document bounded local evidence claims.

**Non-Goals:**
- No new object schema or receipt format.
- No correctness, semantic-equivalence, cross-machine determinism, or exhaustive corpus claim.
- No Aspen/downstream corpus expansion.

## Decisions

### 1. Use a focused Cargo test filter from Nix

**Choice:** Add a `checks.object-corpus-gate` flake check using the existing fingerprint-consumer test harness with an `object_internal_corpus` filter.

**Rationale:** The current test names share that prefix and already exercise the intended receipt replay, fixture freshness, contract metadata, and fresh-target determinism rails. A focused check gives CI a durable target without duplicating Rust logic.

**Alternative:** Add a shell script or new Rust binary. Rejected because the integration tests are already the canonical replay assertions.

## Risks / Trade-offs

**Fixture staleness** → The named check includes the existing fixture-match test, so intentional corpus changes must refresh the checked-in receipt in the same review.

**Overclaiming determinism** → Docs and specs must continue to say bounded local replay/fresh-target evidence, not cross-machine proof.
