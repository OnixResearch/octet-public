## Context

Octet's current adoption story already explains the checked path, public story cards, before/after examples, terminal quickstart, FAQ boundaries, and Salt inspiration limits. The missing artifact is a concrete tutorial-sized proof of use: a reader should be able to inspect one patch-sized Rust example and see exactly how Octet turns it into local review evidence.

Salt's tutorial pattern is useful here: start with one tiny source example, run the tool, show the failure, fix it, and name what was proved or not proved. Octet must translate that into its own evidence vocabulary: lint or policy finding, bounded remediation, artifact sidecars, provenance, artifact verification, and explicit non-claims.

## Approach

1. **Use one high-confidence existing lint family.** Prefer a stable lint with a simple failure shape and small remediation, such as `unchecked_division`, `unchecked_narrowing`, `ambient_clock`, `unbounded_channel`, or `ignored_result`.
2. **Keep the fixture patch-sized.** The walkthrough should fit in one page and avoid building a second consumer guide. If a compilable fixture is added, keep it local to the docs example and use named constants instead of magic numbers.
3. **Show the whole local evidence path.** The command sequence should include `cargo octet check --artifact-dir artifacts/octet` with an output mode that names sidecars, then `cargo octet artifact verify --artifact-dir artifacts/octet`.
4. **Pair every claim with a boundary.** The text should describe deterministic failure shapes and review evidence, not solver models or formal counterexamples unless a scoped proof receipt supports that specific claim.
5. **Validate the page deterministically.** Add or reuse a docs inventory check that asserts required headings, command strings, artifact names, chosen lint name, remediation text, and forbidden unsupported claims.

## Decisions

### 1. Treat the walkthrough as docs/evidence packaging

**Choice:** The change should not alter lint behavior or runner semantics.

**Rationale:** The value is onboarding clarity. Existing Octet surfaces are sufficient for the tutorial.

### 2. Prefer lint evidence over proof evidence

**Choice:** The first walkthrough should use a static lint finding rather than `cargo octet prove`.

**Rationale:** Most new users encounter Octet through `cargo octet check`; proof rails remain optional and scoped. A lint example keeps the path incremental and avoids Salt-style verifier overclaims.

### 3. Make artifact verification visible

**Choice:** The walkthrough should end with artifact verification, not just a clean lint rerun.

**Rationale:** Octet's differentiator is reviewable evidence, provenance, and fail-closed artifact verification. Stopping at diagnostics would undersell the checked path.

## Risks / Trade-offs

- **Example drift:** A selected lint's diagnostic wording may evolve. Mitigate by checking for lint name, command, artifact names, and remediation shape rather than exact stderr text unless a fixture intentionally pins it.
- **Onboarding duplication:** The tutorial may duplicate consumer-guide setup. Mitigate by keeping setup minimal and linking to the full guide for production configuration.
- **Overclaim pressure:** Tutorial copy may read like a proof demonstration. Mitigate with a forbidden-claim check and explicit boundary paragraph.
- **Fixture maintenance:** A runnable docs fixture can add maintenance cost. Mitigate by starting with a small markdown walkthrough unless deterministic checks need a fixture.

## Validation

- Positive documentation inventory confirms the walkthrough contains before code, chosen lint or policy surface, command sequence, fixed code, artifact names, artifact verification, and links to the longer consumer guide.
- Positive command inventory confirms `cargo octet check --artifact-dir artifacts/octet` and `cargo octet artifact verify --artifact-dir artifacts/octet` appear in the walkthrough.
- Positive artifact inventory confirms expected local outputs such as `status.json`, `summary.txt`, findings sidecars, and `provenance.jsonl` are named when selected.
- Negative wording check rejects solver-counterexample, mathematical-certainty, whole-program-proof, certification, C-performance-parity, verified-runtime, and Rust-replacement claims.
- Cairn `validate` and proposal/design/tasks gates pass for this change package.
