## Design

1. Add an explicit optional evidence rail named `z4-solver` in Octet's external evidence model.
2. Define a pure receipt DTO for solver evidence: solver binary/library identity, version/provenance, query hash, logic/profile, result status, proof artifact refs, model/unsat-core refs, timeout/unknown reason, validation policy, diagnostics, and non-claims.
3. Treat `unknown` as a first-class non-proof result. `unknown`, timeout, and unsupported logic may be useful diagnostics but cannot satisfy proof obligations.
4. Treat solver outputs as recorded-only unless a later rail includes an independent checker for the exact proof format and input identity.
5. Keep solver execution, file reads, process spawning, timeout enforcement, and raw log capture in `cargo-octet` shell/collector code, not lint or pure validation cores.
6. Add positive fixtures for satisfiable diagnostic output, unsat-with-proof recorded output, and conservative unknown output.
7. Add negative fixtures for stale query hash, unsupported logic promoted as pass, mismatched solver identity, missing proof ref for proof claim, stale proof artifact, and broad Z3-replacement claim.

## Non-Goals

- No mandatory z4 dependency or default check integration.
- No replacement for Verus, Kani, Z3, or existing proof rails.
- No claim that Octet proves z4 soundness, solver completeness, or broad SMT-LIB compatibility.