## Context

`standards/policies/registry.ncl` already declares `tigerstyle.architecture.effect-capability-boundaries` with `current_enforcement_mode = "lint"`, `target_enforcement_mode = "structural"`, and a `capability-api` mechanism. `architecture.rs` also has a hard-coded core scan for ambient authority patterns. That proves the checker idea, but the policy artifact seam does not yet carry the actual capability obligations, and fixtures cannot demonstrate that the generated bundle is the enforcement source of truth.

## Goals / Non-Goals

**Goals:**
- Put capability-api checker obligations in typed Nickel contracts and generated artifacts.
- Consume those obligations in Rust architecture checks rather than relying only on hard-coded ambient patterns.
- Prove the gate with Tigerstyle-local positive/negative fixtures and artifact-backed checks.

**Non-Goals:**
- No Aspen or other downstream consumer mutation proof in this slice.
- No full Rust AST/HIR analysis yet; source-pattern checks are acceptable if their obligations come from policy artifacts and fixtures cover the contract.
- No attempt to classify every project-specific capability trait name; the first gate only proves forbidden ambient constructors and approved shell/core shapes.

## Decisions

### 1. Capability-api mechanism obligations are artifact fields

**Choice:** Add mechanism fields for `forbidden_ambient_apis`, `core_source_scopes`, `shell_constructor_scopes`, and `approved_capability_names`.

**Rationale:** Nickel remains the typed policy authoring surface while Rust remains the checker. This matches the bounded-queue structural gate pattern and avoids reintroducing hidden Rust-only policy constants.

**Alternative:** Keep the current hard-coded pattern list in `architecture.rs`. Rejected because generated policy artifacts would claim structural intent without carrying the enforcement obligations.

### 2. Structural diagnostics stay scoped to declared architecture roots

**Choice:** The first checker only evaluates source paths matching declared core scopes for forbidden ambient operations, and treats shell constructor scopes as allowed construction sites.

**Rationale:** Effect-capability boundaries are about core/shell placement. Source-scope obligations keep false positives low and make fixture proof deterministic.

**Alternative:** Scan every Rust file in the repository. Rejected because shell code is supposed to construct ambient capabilities.

### 3. Tigerstyle-local proof replaces downstream mutation proof

**Choice:** Evidence for this slice uses Tigerstyle architecture fixtures, generated artifact export assertions, and `tigerstyle-standards check --policy-artifacts-dir` reports.

**Rationale:** The user explicitly wants to keep working on Tigerstyle instead of Aspen testing. Local fixtures are faster, deterministic, and enough to prove the checker seam.

## Risks / Trade-offs

- **Pattern false positives:** a textual pattern may match comments or declarations. Mitigate by fixture coverage and small line filters; deeper parsing can be a later OpenSpec.
- **Under-specified capability names:** approved capability names are documentation/evidence initially; the hard fail focuses on forbidden ambient constructors in core.
- **Artifact schema churn:** adding mechanism fields changes generated JSON. Mitigate with model round-trip tests and generated artifact assertions.

## Validation Plan

1. Typecheck Nickel contracts and registry with capability-api obligations.
2. Assert generated `tigerstyle-policies.json` includes the new capability obligation fields for `tigerstyle.architecture.effect-capability-boundaries`.
3. Run architecture fixtures proving compliant injected capability code passes and noncompliant core ambient authority fails.
4. Run focused Rust tests/clippy and Nix standards/docs checks.
5. Save evidence under this change before marking tasks complete.
