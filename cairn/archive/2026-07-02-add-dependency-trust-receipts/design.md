## Context

`cargo-deny` and vulnerability scanners evaluate known policy data. They do not record whether humans reviewed third-party dependency code. `cargo-vet` provides a Rust-native audit/exemption/import workflow that Octet can record as local review evidence.

## Approach

1. **Optional cargo-vet rail.** Add execution or import support for `cargo vet` results.
2. **Trust receipt DTO.** Record audit coverage, criteria, imports, exemptions, gaps, command argv, config root, and review actions.
3. **Evidence tier selection.** Fully covered configured criteria can produce `policy_check`; gaps, exemptions, or deferred audits produce `manual_review_required` unless policy says they block.
4. **Separate summaries.** Dependency trust appears separately from dependency audit/advisory results.

## Decisions

### Keep cargo-vet social model outside Octet

**Choice:** Octet records imported audit identities but does not operate a trust network.

**Rationale:** Local-first evidence needs reviewable inputs, not a new hosted service.

### Exemptions are visible

**Choice:** Exemptions and deferred audits always appear in receipts.

**Rationale:** A green summary that hides unaudited allowances would undermine Octet's evidence chain.

## Risks / Trade-offs

- **Human audit quality:** Octet cannot prove audit quality. Mitigate with explicit non-claims and manual-review actions.
- **Import trust:** Imported audits can be stale or from weak sources. Mitigate by recording import identity and freshness data when available.
- **Adoption friction:** `cargo-vet` setup is non-trivial. Mitigate by making the rail optional and documenting starter rollout.

## Validation

- Fully-covered fixture emits successful receipt.
- Unaudited gap fixture emits manual-review or hard-fail outcome.
- Exemption fixture preserves package/rationale/review action.
- Import identity missing fails closed.
- Cairn validate and gates pass for this change.
