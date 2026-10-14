## Context

The production pilot currently binds a real Verus release, Rust toolchain, bundled solver, command argv, source/proof paths, pass signal, and Nix closure graph digest. Focused validation shows the happy path can produce valid scoped `formal_proof` provenance for `src/verified/addition.rs` and `verus/proofs/addition_bound.rs`.

The remaining gap is negative evidence. A production rail must demonstrate that an invalid proof is rejected by real Verus and that Octet's receipt/provenance path treats the failed verifier result as non-proof evidence only.

## Approach

1. **Use a deliberately invalid Verus proof target.** Add a tiny proof file whose postcondition cannot be established by Verus. Keep it isolated from the positive proof target and document that it exists only for failure-path verification.
2. **Exercise the real wrapper.** The direct negative check should invoke `octet-production-verus` rather than the fixture `octet-verus` wrapper so the test covers production verifier behavior.
3. **Route failure through `cargo octet prove`.** Add a selected negative rail or focused fixture manifest so the CLI path captures bounded stdout/stderr, records a deterministic failure diagnostic, and exits non-zero.
4. **Keep validation pure.** Receipt/provenance rejection logic remains in deterministic proof-validation functions over typed command results and in-memory receipt/provenance inputs. CLI code remains the imperative shell for running Verus and reading/writing artifacts.
5. **Assert absence, not just failure.** Tests must verify no `formal_proof` provenance is emitted for the failed production rail, even if stdout/stderr contain misleading proof-like text.

## Decisions

### 1. Negative proof lives beside positive proof fixtures

**Choice:** Put the failing proof under `verus/proofs/` with a clear name such as `addition_bound_invalid.rs` or equivalent.

**Rationale:** Keeping it near the positive pilot makes the proof relationship auditable while namespacing prevents accidental use as a valid rail.

### 2. Failure rail is test-scoped unless product docs need to expose it

**Choice:** Prefer a test fixture manifest or clearly marked negative rail over adding a normal operator-facing production rail that always fails.

**Rationale:** Operators should not see an always-failing production rail in normal `cargo octet prove --list` output unless they intentionally select failure-path tests.

### 3. Diagnostics name verifier failure class

**Choice:** The failing path should report a deterministic verifier-exit diagnostic and artifact locations, not a generic receipt mismatch.

**Rationale:** Operators need to distinguish proof failure from stale receipt, manifest, or provenance corruption.

## Risks / Trade-offs

- **Fixture pollution:** A checked-in invalid proof can confuse readers. Mitigate with explicit filename/comment/docs that it is failure-path evidence only.
- **Verus output churn:** Verifier diagnostics may change between pinned versions. Mitigate by asserting stable Octet diagnostics and high-level non-zero status, not full raw Verus stderr text.
- **Normal rail selection:** An always-failing rail could break default proof runs. Mitigate by keeping the negative rail fixture-scoped or opt-in only.

## Validation

- Direct negative wrapper run exits non-zero for the invalid proof.
- `cargo octet prove` with the negative production fixture exits non-zero and writes bounded failure artifacts.
- Artifact/provenance assertions show no `formal_proof` provenance for the failed rail.
- Existing positive production and fixture proof rails still pass.
