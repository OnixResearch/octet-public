## Context

Octet currently presents several high-assurance surfaces: Dylint lints, standards registry checks, receipt validation, Verus proof rails, proof cards, provenance, artifact verification, and release smoke transcripts. A reader can inspect each surface, but there is no single concept that explains how a source-level safety expectation becomes progressively stronger evidence.

A checked-contract ladder gives Octet a contract-first story while staying honest about its implementation model. Unlike a new language with compiler-native preconditions, Octet operates on Rust source and release artifacts. Its contract unit is therefore not a syntax form; it is a scoped claim tied to evidence and a gate.

## Approach

1. **Define the ladder.** Document the ordered assurance stages as advisory lint, warning gate, deny gate, policy requirement, receipt-backed validation, scoped formal-proof receipt, and verified release artifact.
2. **Define checked-contract fields.** A checked contract should name a requirement or policy ID, source scope, invariant or failure shape, evidence tier, command, artifact/receipt path, reviewer action, and explicit non-claim.
3. **Map existing surfaces.** Show how current lint diagnostics, standards checks, Verus proof cards, `formal_proof` provenance, and artifact verification populate the ladder without changing behavior.
4. **Improve diagnostic expectations.** For high-confidence lints, specify that diagnostics should name the failing shape and remediation boundary when that can be derived deterministically. This is a UX contract, not an obligation to solve arbitrary path conditions.
5. **Keep future implementation split.** Treat this change as the vocabulary/spec layer. Follow-up changes can add machine-readable contract manifests, richer diagnostics, or additional proof bindings.

## Decisions

### 1. Checked contracts are evidence records, not syntax

**Choice:** Define checked contracts as scoped requirement/evidence records instead of Rust syntax extensions.

**Rationale:** Octet must remain compatible with existing Rust workspaces and avoid creating a parallel language. The durable unit reviewers need is the claim-to-evidence mapping.

### 2. The ladder is progressive, not automatically monotonic proof

**Choice:** The ladder can move a claim from lint advice to release evidence, but each stage must name its evidence tier and limitation.

**Rationale:** A deny-level lint is stronger operationally than an advisory warning, but it is not the same as formal proof. The ladder must improve adoption without collapsing evidence categories.

### 3. Diagnostics should explain failure shape when deterministic

**Choice:** Require high-confidence diagnostic UX to name the invalid state, branch, boundary, or missing guard when available from Octet's analysis.

**Rationale:** Salt's counterexample story is compelling because failures feel concrete. Octet can adapt that lesson by making lint findings more specific without implying SMT counterexamples.

### 4. Machine-readable contract manifests are follow-up work

**Choice:** This change should define the fields and constraints, but not force a new manifest format.

**Rationale:** If a machine-readable manifest is introduced later, repo guidance says human-authored config should use Nickel and runtime receipts should be Rust-derived. That design deserves its own change.

## Risks / Trade-offs

- **Terminology risk:** "Contract" can imply compiler-native preconditions. Mitigate by defining checked contracts as evidence records and naming non-goals.
- **Scope creep:** The ladder could expand into new verifier/runtime behavior. Mitigate by making this a docs/spec change and splitting implementation follow-ups.
- **Evidence confusion:** Readers may treat deny gates as proof. Mitigate by requiring evidence-tier labels at each ladder stage.
- **Diagnostic overreach:** Counterexample-style language could imply SMT. Mitigate by saying diagnostics name deterministic failure shapes, not arbitrary model counterexamples.

## Validation

- Positive documentation inventory confirms every assurance-ladder stage is named with its evidence tier, command or gate, and artifact/receipt surface where applicable.
- Positive checked-contract inventory confirms every required field is defined and mapped to existing Octet surfaces.
- Positive diagnostic UX check confirms high-confidence finding examples name the failure shape and remediation boundary without claiming SMT counterexamples.
- Negative wording check rejects language-level contract syntax claims, solver-completeness claims, runtime-fallback claims, whole-program proof claims, and statements that all ladder stages are formal proof.
- Cairn `validate` and proposal/design/tasks gates pass for this change package.
