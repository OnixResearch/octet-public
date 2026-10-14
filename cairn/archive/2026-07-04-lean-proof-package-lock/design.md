## Context

Existing Verus proof rail requirements already pin verifier identity and structured receipts. A proof package lock is the reviewable bundle boundary around those facts: it names the exact material a proof run intends to use before execution and the material a saved receipt claims afterward.

## Decisions

### 1. Lock proof material before promotion

**Choice:** Selected production proof rails SHOULD produce or consume a deterministic lock record that binds manifest, source, proof, verifier, solver, toolchain, closure, command, environment, pass signal, and trust-boundary identities.

**Rationale:** Operators need one offline artifact that says what proof package is being checked, not scattered identity fragments.

### 2. Validate locks offline and fail closed

**Choice:** Lock validation MUST use checked-in files, declared paths, BLAKE3 identities, and Nix closure material already available locally. It MUST NOT fetch network schemas, packages, or tool metadata.

**Rationale:** Release evidence and replay should be deterministic and usable in offline CI or release environments.

### 3. Keep lock and receipt roles distinct

**Choice:** A lock states intended proof material. A receipt states an executed verifier result. Formal-proof provenance still requires a validated receipt matching the lock.

**Rationale:** A correct lock is not a proof that Verus succeeded.

### 4. Preserve starter-path independence

**Choice:** Proof package locks apply only to selected proof rails and MUST NOT be required for starter lint checks or ordinary `cargo octet check` runs.

**Rationale:** Proof rails are opt-in and should not raise the adoption cost of the basic checked path.

## Risks / Trade-offs

- Lock records can duplicate receipt fields; keep shared identity helpers and clear role separation.
- Closure graph identity can be expensive; use existing canonical closure helpers and named validation budgets.
- A lock must not imply solver soundness, proof success, or whole-program correctness.
