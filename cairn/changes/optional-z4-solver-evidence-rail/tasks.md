## Tasks

- [ ] [serial] r[octet.experimental_solver_rails.z4_solver.profile] Define the optional z4-solver evidence rail profile and receipt DTO.
- [ ] [depends:z4-rail-profile] r[octet.experimental_solver_rails.z4_solver.validation] Validate solver identity, query hash, logic profile, result status, proof/model/core refs, unknown reasons, diagnostics, roles, and non-claims.
- [ ] [depends:z4-rail-validation] r[octet.experimental_solver_rails.z4_solver.unknown] Preserve `unknown` and timeout as non-proof diagnostic outcomes.
- [ ] [depends:z4-rail-validation] r[octet.experimental_solver_rails.z4_solver.positive] Add positive fixtures for SAT diagnostics, UNSAT-with-proof recorded output, and UNKNOWN output.
- [ ] [depends:z4-rail-validation] r[octet.experimental_solver_rails.z4_solver.negative] Add negative fixtures for stale query hash, unsupported-logic promotion, mismatched solver, missing proof ref, stale proof artifact, and replacement overclaim.
- [ ] [depends:z4-rail-fixtures] r[octet.experimental_solver_rails.z4_solver.shell] Keep execution, timeouts, logs, filesystem reads, and presentation in collector shell code.
- [ ] [depends:z4-rail-shell] r[octet.experimental_solver_rails.z4_solver.final_validation] Run rail fixture tests, artifact verification, proof-card checks, Cairn gates, and focused Octet checks.