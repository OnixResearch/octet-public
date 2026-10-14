# Design: Add Hegel property evidence rail

## Approach

The Hegel rail is an optional external evidence rail. The CLI shell accepts a declared Hegel result input and normalized run metadata, while pure receipt validation checks the typed in-memory receipt before artifact verification accepts it. Starter `cargo octet check` must not invoke Hegel or require Hegel artifacts.

## Receipt shape

A Hegel receipt records:

- rail id, schema version, and Octet policy identity;
- property ID, linked requirement or policy IDs, checked package/module/function family, and source scope;
- Hegel crate version, backend/native mode, runner settings, case budget, phases, seed or derandomized mode, and CI/database posture;
- generator family identity and BLAKE3 generator/content hash where applicable;
- status, issue codes, minimal counterexample summary hash for failures, and bounded log identity;
- parent artifacts when the Hegel receipt summarizes a fixture corpus or pure-core law catalog;
- explicit non-claims.

## Evidence classification

Passing Hegel runs are scoped generated-testing evidence. Failing runs are preserved as reviewable diagnostics and may seed checked-in regression fixtures, but they are not passing evidence. Hegel receipts must never promote to `formal_proof`; Valence exports preserve their generated-testing tier.

## Pure-core guidance

Use Hegel for deterministic pure cores such as config merging, artifact verification decisions, fingerprint review identity, proof-ref validation, migration manifests, and standards policy checkers. Do not use Hegel as a replacement for Dylint UI fixtures, CLI golden output, artifact verification, or transcript evidence where user-visible behavior is the contract.

## Fixture strategy

Positive fixtures cover a complete passing receipt, deterministic generated settings, a property linked to a requirement ID, and a Valence export record. Negative fixtures cover missing runner identity, missing source scope, stale generator hash, malformed BLAKE3 digest, failing property status presented as passing evidence, unsupported evidence-tier promotion, and missing non-claims.

## Cross-repo boundary

This change depends on Valence accepting bounded Hegel property receipt semantics. Octet owns collection, artifact verification, and Valence export of Octet-produced Hegel facts; Valence owns the shared IR receipt boundary and stack-side non-claim enforcement.
