## Context

The verified rail currently treats a receipt as valid when it has a pass signal plus configured BLAKE3 source/command binding and provenance lines. That proves the receipt content is tied to the checked source and command, but the receipt does not yet account for the verifier/tooling identity or toolchain identity expected by policy.

## Goals / Non-Goals

**Goals:**
- Model verifier/toolchain receipt obligations in typed Nickel policy.
- Keep the generated artifact seam deterministic.
- Make receipt-only validation reject passing receipts missing required verifier/toolchain lines.
- Make `--execute-proof-rails` generate those lines from configured metadata.

**Non-Goals:**
- Execute remote verifiers or fetch tool metadata from the network.
- Introduce credential-dependent proof execution.
- Replace source/command hash binding or receipt provenance requirements.

## Decisions

### 1. Add a nested `receipt_verifier` proof rail contract

**Choice:** Add `ReceiptVerifierContract` with `verifier_id`, `verifier_digest`, `toolchain_id`, and `toolchain_digest`, each defaulting to the empty string.

**Rationale:** This keeps policy authoring typed and opt-in while allowing future rails to choose the identity/digest source appropriate to their verifier backend. Empty defaults preserve existing rails.

**Alternative:** Infer verifier/toolchain metadata from the local process on every read-only check. Rejected because receipt-only validation must be deterministic from policy artifacts and checked files, not from ambient host state.

**Implementation:** Rust models mirror the Nickel contract, generated artifacts export all four fields, and proof receipt validation requires matching line-oriented metadata only when fields are configured.

### 2. Use line-oriented receipt metadata

**Choice:** Require exact receipt lines:

```text
proof-verifier: <verifier_id>
proof-verifier-blake3: <verifier_digest>
proof-toolchain: <toolchain_id>
proof-toolchain-blake3: <toolchain_digest>
```

**Rationale:** Existing receipts are line-oriented, easy to inspect in archives, and easy for downstream consumers to generate without linking to Rust internals.

**Alternative:** Embed a nested JSON receipt block. Rejected for this slice because it would churn the simple proof fixture format before multiple backends need richer typed receipts.

### 3. Preserve local deterministic command execution

**Choice:** `--execute-proof-rails` emits the configured verifier/toolchain lines alongside existing provenance and binding lines, without probing network, credentials, or mutable host state.

**Rationale:** The current rail already requires proof command execution to be local/deterministic. Policy-authored verifier/toolchain fields make the receipt accountable without adding nondeterministic ambient dependencies.

## Risks / Trade-offs

**Configured digest drift** → Mitigate with generated artifact assertions and docs explaining that verifier/toolchain digest values are policy obligations and must be updated when the verifier/toolchain contract changes.

**Metadata-only false confidence** → Mitigate by keeping source/command hash binding and pass-signal/provenance validation mandatory for the current verified rail.
