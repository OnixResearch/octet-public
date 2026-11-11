# bind-proof-rail-receipt-hashes Design

## Context

The existing proof rail checker treats the pass signal as the hard gate. `--execute-proof-rails` synthesizes receipts from local command output, but static and generated receipts need a deterministic binding contract.

## Decisions

### 1. BLAKE3 receipt lines

**Choice:** Require receipt lines of the form `proof-command-blake3: <hex>` and `proof-source-blake3: <path>:<hex>` when a proof rail opts into receipt binding.

**Rationale:** BLAKE3 is already present in the workspace and yields compact deterministic content hashes. The explicit line prefix is easy to inspect and easy for the Rust checker to validate without parsing a complex receipt format.

### 2. Source-root-relative paths

**Choice:** Hash lines use the proof rail's configured source path patterns, matched against checked source paths, and store the matched source path text next to the hash.

**Rationale:** This preserves the existing generated-artifact seam and avoids absolute checkout paths in receipts.

### 3. Opt-in policy field

**Choice:** Add `receipt_binding = "blake3-command-and-source"` to proof rails, with `"none"` as the compatibility default.

**Rationale:** Existing metadata remains readable, while hard verified rails can demand stronger receipt provenance.

## Risks / Trade-offs

- Static receipt fixtures must be refreshed whenever covered source changes. Mitigation: command-generated receipts are preferred for verification fixtures, and static receipts are small and deterministic.
- Path matching remains pattern based. Mitigation: keep configured paths source-root-relative and add regression tests for the generated receipt content.
