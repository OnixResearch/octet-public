## Phase 1: Implementation

- [x] [serial] r[octet.valence_project_extraction.phase4.readiness_inventory] Update `valence-core` README and crate-level docs so they enumerate every public module and each module's boundary.
- [x] [serial] r[octet.valence_project_extraction.phase4.readiness_gate] Extend `valence-core-package-readiness` to assert module inventory agreement, required docs/non-claims, focused tests, API docs, and package listing.
- [x] [serial] r[octet.valence_project_extraction.phase4.non_claims] Preserve the in-repository/no-publication/no-cutover boundary in docs and gate output.

## Phase 2: Verification

- [x] [serial] r[octet.valence_project_extraction.phase4.readiness_inventory.in_sync] Verify the positive inventory case names `evidence_index`, `function_object`, and `evidence_codebase_index` consistently across README, crate docs, and public exports.
- [x] [serial] r[octet.valence_project_extraction.phase4.readiness_inventory.fail_closed] Verify negative missing-module and stale-module fixtures fail the readiness assertion without passing silently.
- [x] [serial] r[octet.valence_project_extraction.phase4.readiness_gate.named] Run `cargo test -p valence-core`, `cargo doc -p valence-core --no-deps`, `cargo package -p valence-core --allow-dirty --no-verify --list`, and `nix build .#checks.x86_64-linux.valence-core-package-readiness --no-link -L`.
- [x] [serial] r[octet.valence_project_extraction.phase4.non_claims.no_cutover] Verify the readiness output and docs still state no publication, no standalone repository cutover, and no correctness/semantic-equivalence/general-HIR-THIR-MIR claims.
- [x] [serial] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout. [depends:cairn-gates]
