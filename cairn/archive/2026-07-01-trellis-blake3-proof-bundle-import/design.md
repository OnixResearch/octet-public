## Context

Trellis provides useful reference surfaces for Octet's Verus evidence work:

- `../trellis/cairn-policy/contracts.ncl` defines a BLAKE3 receipt-hash policy with deterministic JSON canonicalization.
- `../trellis/src/ucan_proof_reference.rs` verifies digest admission over shell-supplied proof-reference bytes: fixed-width digest, exact equality, wrong-length rejection, and mismatch rejection.
- `../trellis/src/chain.rs` and `../trellis/src/snapshot_integrity.rs` model BLAKE3-style hashing as uninterpreted specs with explicit trusted runtime and collision-resistance boundaries.
- `../trellis/scripts/verus.sh` pins a Verus wrapper path and verifier invocation shape for the Trellis corpus.
- `../trellis/proof-review/policy.ncl` and generated policy JSON show proof-review scoring and validation-command contracts.

Octet should use these as reference patterns, not as ambient proof authority.

## Approach

1. **Define an import manifest contract.** The human-authored manifest should be Nickel with typed contracts for repository label, repository revision, source artifacts, proof artifacts, command argv, verifier identity fields, BLAKE3 hash fields, trust-boundary notes, and intended evidence tier.
2. **Export deterministic runtime input.** Runtime Rust should consume generated JSON, not execute Nickel dynamically. The generated JSON should be covered by freshness checks and artifact manifest rules.
3. **Separate identity from proof.** BLAKE3 fields prove exact byte identity for source, proof, logs, and receipts. They do not prove the Verus result, solver soundness, model adequacy, or Octet receipt validity.
4. **Add import validation core.** A pure validation core should check required fields, supported hash algorithm, digest shape, duplicate identities, stale hashes, source/proof path normalization, and evidence-tier rules over in-memory data.
5. **Keep the shell thin.** CLI or artifact verifier code reads files, computes BLAKE3 hashes, loads generated JSON, and passes typed inputs to the validation core.
6. **Promote only through rerun.** An imported Trellis bundle may become Octet `formal_proof` only after Octet reruns the selected proof with pinned verifier/solver/toolchain identity and validates an Octet-owned structured receipt.

## Decisions

### 1. Imported bundle is a reference, not a receipt

**Choice:** Give imported Trellis bundles their own evidence-origin value and keep them non-`formal_proof` by default.

**Rationale:** The authoritative Octet formal-proof path requires Octet-owned verifier identity, command capture, receipt validation, and provenance emission.

### 2. Nickel owns the human-authored import manifest

**Choice:** Keep the curated list of Trellis references in typed Nickel and export deterministic JSON.

**Rationale:** Humans will select and update imported proof references. Nickel contracts catch missing fields and unsupported evidence-tier choices before runtime execution.

### 3. Reuse Trellis digest-admission ideas, not its exact proof model

**Choice:** Adapt the fixed digest/admission/mismatch behavior from `ucan_proof_reference.rs` for Octet import validation.

**Rationale:** Octet needs byte-identity checks for artifacts. It does not need to import Trellis's BLAKE3 uninterpreted proof model into Octet's verifier rails for this change.

## Risks / Trade-offs

- **External drift:** Trellis paths or revisions can drift. Mitigate by binding repository revision and BLAKE3 artifact hashes.
- **Overclaiming:** Imported proof bundles can look like Octet formal proof. Mitigate by fail-closing any imported-as-`formal_proof` card or provenance without rerun receipt.
- **Manifest burden:** Curated imports require maintenance. Mitigate by keeping the first fixture small and focused on one or two Trellis proof artifacts.
- **Trust-boundary confusion:** Trellis has explicit BLAKE3 trust axioms. Mitigate by preserving those notes as trust-boundary text, not hiding them in a hash field.

## Validation

- Positive import fixture validates repository revision, normalized paths, hash algorithm, BLAKE3 artifact identities, trust-boundary notes, and external/manual-review tier.
- Positive rerun-path fixture shows the imported bundle can name the Octet rerun command required for promotion without claiming that promotion happened.
- Negative fixtures reject missing required fields, malformed digest shape, stale source/proof/log hash, duplicate artifact identity where uniqueness is required, unsupported hash algorithm, and imported-as-`formal_proof` without Octet receipt.
- Policy freshness and registry checks cover the Nickel manifest, generated JSON, and config/receipt family classification.
- Cairn validation and gates pass for this change package before implementation and before archive.
