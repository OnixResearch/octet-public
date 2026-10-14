## Contract

- [x] [serial] Specify the initial cross-gate provenance producer set and required source ID families. r[octet.evidence_provenance.cross_gate_coverage]
- [x] [serial] Specify known checker/source registry validation for provenance producers. r[octet.evidence_provenance.producer_registry]
- [x] [serial] Specify parent artifact references, missing-parent rejection, and cycle rejection for derived evidence chains. r[octet.evidence_provenance.parent_chain]

## Implementation

- [x] [serial] Add producer registry data and pure validation for checker IDs, source families, parent references, and parent cycles. r[octet.evidence_provenance.producer_registry]
- [x] [serial] Extend standards, policy, fingerprint, and transcript outputs to emit or validate provenance records for deterministic artifacts. r[octet.evidence_provenance.cross_gate_coverage]
- [x] [serial] Update release/operator docs with a multi-gate provenance inspection path and tier boundaries. r[octet.evidence_provenance.parent_chain]

## Verification

- [x] [serial] Add positive fixtures for a multi-gate provenance chain covering runner, standards/policy, fingerprint, and transcript artifacts. r[octet.evidence_provenance.cross_gate_coverage]
- [x] [serial] Add negative fixtures for unknown producer, missing parent, parent-cycle, hash mismatch, invalid tier assignment, and formal-proof overclaim. r[octet.evidence_provenance.parent_chain]
- [x] [serial] Run targeted provenance tests, standards/policy tests, docs checks, and Cairn validation/gates. r[octet.evidence_provenance.producer_registry]
