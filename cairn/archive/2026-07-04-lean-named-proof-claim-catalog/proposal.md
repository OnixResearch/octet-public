## Why

Lean libraries are useful because important facts have stable names. Octet has requirement IDs, policy IDs, proof-card IDs, checker IDs, and evidence source IDs, but there is not yet a single claim-catalog contract that says which named proof or evidence claim is stable enough to reference from docs, proof cards, future suppressions, or release evidence.

Octet should adapt Lean's named theorem inventory as a boring Rust/Nickel catalog: stable IDs, explicit scopes, aliases for renames, and fail-closed validation for stale or overbroad references.

## What Changes

- Define a proof-claim catalog capability for stable claim IDs that can link requirements, policies, proof cards, proof refs, evidence receipts, and documentation anchors.
- Require unique IDs, explicit source scope, evidence tier expectations, non-claims, owner surface, and lifecycle status.
- Add rename/deprecation alias handling so references do not silently drift.
- Add positive and negative validation for duplicate IDs, missing linked artifacts, stale aliases, invalid evidence-tier promotion, and unsupported correctness claims.

## Impact

- **Files**: likely `standards/policies/` Nickel catalog inputs, generated policy artifacts, proof-card/proof-ref validation cores, docs for proof and evidence references, and fixtures for catalog validation.
- **Testing**: catalog export/freshness tests, positive linked-claim fixtures, negative duplicate/stale/missing/overclaim fixtures, proof-card/proof-ref compatibility tests, and Cairn validate/gates.
