## Context

The verified proof rail checker already requires pass-signal, source/command hash binding, provenance/trust-boundary metadata, and configured verifier/toolchain identity digests. The remaining gap is that the configured verifier identity can be reused by a different executable unless the receipt also names and hashes the verifier binary/build artifact.

## Decisions

### 1. Extend receipt verifier metadata rather than add a new proof rail object

**Choice:** Add `verifier_binary_path` and `verifier_binary_digest` to the existing receipt verifier contract/model.

**Rationale:** Binary binding is part of verifier identity, and keeping it in the same nested contract preserves the Nickel-authored policy seam and generated artifact shape.

**Rejected:** A separate receipt binary binding object. That would fragment one verifier obligation across multiple optional objects without adding enforcement value.

### 2. Generated receipts compute the running executable digest

**Choice:** `--execute-proof-rails` emits `proof-verifier-binary` and `proof-verifier-binary-blake3` from `std::env::current_exe()` when binary metadata is configured.

**Rationale:** Command-generated receipts should attest the actual local verifier executable used to create the receipt, not copy a free-form configured string.

**Risk:** The running binary digest may differ from a stale configured digest. That is intentional; the generated receipt will then fail until policy metadata is refreshed.

### 3. Receipt-only checks remain deterministic string matching

**Choice:** Default receipt-only checks compare receipt lines against policy-configured binary path/digest metadata.

**Rationale:** Receipt-only validation may not have access to the original executable, so the proof receipt must carry the digest while policy artifacts declare the expected value.

## Validation Plan

- Add positive static fixture metadata and missing-binary-metadata regression.
- Assert generated artifact export includes verifier binary fields.
- Run focused cargo tests, clippy, policy validation, standards fixtures, docs examples, OpenSpec validation, and whitespace checks.
