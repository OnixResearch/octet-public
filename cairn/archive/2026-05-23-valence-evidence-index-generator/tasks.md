## Phase 1: Implementation

- [x] [serial] r[octet.valence_evidence.index_generator.render] Add a deterministic generator for `docs/current-head-valence-evidence.md` from payload commit, rail inventory, pass signals, and bounded non-claim text.
- [x] [serial] r[octet.valence_evidence.index_generator.check] Add a fail-closed check mode that compares generated output with the checked-in Markdown page and reports stable mismatch classes.
- [x] [serial] r[octet.valence_evidence.index_generator.negative] Add negative coverage for stale payload commit, missing rail/pass signal, and changed non-claim boundaries.
- [x] [serial] r[octet.valence_evidence.index_generator.nix_gate] Wire the generator/checker into a named Nix check and include it in the local evidence closeout path.

## Phase 2: Verification

- [x] [serial] r[octet.valence_evidence.index_generator.render.matches_current] Verify the generator output matches the current committed evidence page for the audited payload commit.
- [x] [serial] r[octet.valence_evidence.index_generator.check.fail_closed] Verify the negative fixtures fail closed with deterministic mismatch issue codes.
- [x] [serial] r[octet.valence_evidence.index_generator.nix_gate.aggregate] Verify the named Nix check and `valence-object-evidence` both pass.
- [x] [serial] r[octet.valence_evidence.index_generator.nix_gate] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout.
