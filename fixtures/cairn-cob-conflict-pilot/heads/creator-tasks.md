## Implementation

<!-- conflict-fixture-candidate: creator -->

- [ ] [serial] I1 Create creator/delegate same-basis signed divergence and explicit no-remote Git transport evidence. r[octet.cob_conflict_recovery.divergence] r[octet.cob_conflict_recovery.divergence.same_basis] r[octet.cob_conflict_recovery.divergence.transport]
- [ ] [depends:I1] I2 Retain metadata-only and exact-selector conflict inspections plus ordinary-update refusal evidence. r[octet.cob_conflict_recovery.inspection] r[octet.cob_conflict_recovery.inspection.metadata] r[octet.cob_conflict_recovery.inspection.values] r[octet.cob_conflict_recovery.refusal] r[octet.cob_conflict_recovery.refusal.update]
- [ ] [depends:I2] I3 Preview and execute complete-basis resolution, then reload and checkout from a third store. r[octet.cob_conflict_recovery.resolution] r[octet.cob_conflict_recovery.resolution.preview] r[octet.cob_conflict_recovery.resolution.execute] r[octet.cob_conflict_recovery.reload] r[octet.cob_conflict_recovery.reload.status] r[octet.cob_conflict_recovery.reload.checkout]
- [ ] [depends:I3] I4 Add the typed manifest, deterministic positive/negative Rust validator, policy registry entries, documentation, and hermetic Nix gate. r[octet.cob_conflict_recovery.evidence] r[octet.cob_conflict_recovery.evidence.positive] r[octet.cob_conflict_recovery.evidence.negative] r[octet.cob_conflict_recovery.boundary] r[octet.cob_conflict_recovery.boundary.non_claims]

## Validation

- [ ] [depends:I4] V1 Run focused Rust tests, Nickel freshness, Git bundle/ref/ancestry validation, formatting, release-floor Clippy, Octet policy validation, and the dedicated Nix gate. r[octet.cob_conflict_recovery.evidence.positive] r[octet.cob_conflict_recovery.evidence.negative]
- [ ] [depends:V1] V2 Run native Cairn proposal, design, tasks, strict validation, sync, traceability, and deterministic archive gates; report inherited baseline failures separately. r[octet.cob_conflict_recovery.boundary] r[octet.cob_conflict_recovery.boundary.non_claims]
- [ ] [depends:V2] V3 Run Nextest and the broad flake gate, retaining exact pass or bounded blocker evidence before archive. r[octet.cob_conflict_recovery.evidence.positive]
