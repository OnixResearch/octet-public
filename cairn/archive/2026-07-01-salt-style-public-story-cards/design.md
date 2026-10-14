## Context

The accepted adoption-story and product-positioning specs already require Octet to present a Rust-compatible, evidence-backed checked path with reviewable case studies and scoped claim boundaries. The missing piece is packaging: a reader should be able to skim one surface and understand what Octet does, what artifacts it emits, which evidence-backed systems have used it, and which claims remain out of scope.

Salt's presentation offers a useful pattern, but Octet's substance is different. Octet is not a new language and does not embed SMT contracts in Rust compilation. The design therefore adapts only the reader journey and evidence layout.

## Approach

1. **One canonical story surface.** Extend the adoption-story page or create a linked public story section that owns the card inventory. README links to it rather than duplicating the full content.
2. **Cards over prose walls.** Use short sections/cards for the checked path, how it works, built-with evidence, before/after examples, quickstart, and FAQ.
3. **Examples grounded in existing lints.** Show high-confidence remediations for lints that already have clear failure shapes: unchecked division, unchecked narrowing, unbounded channels/collection growth, ambient clock in core, public unsafe API, and ignored results.
4. **Terminal-first quickstart.** Keep the current setup contract but show the expected command sequence and artifact outputs, including `cargo octet check --artifact-dir ...` followed by artifact verification.
5. **FAQ as boundary reinforcement.** Use short answers to common comparison questions while repeating Octet's exact non-claims.
6. **Deterministic doc checks.** Add a small inventory check for headings/topics/examples and a forbidden-claim check for high-risk wording.

## Decisions

### 1. Reuse adoption-story as the canonical source

**Choice:** The card surface should live in or be linked from `docs/adoption-story.md`.

**Rationale:** The accepted spec already defines case-study and methodology contracts. Keeping the presentation work there avoids a second claim surface.

### 2. Treat examples as lint remediation, not formal counterexamples

**Choice:** Before/after examples should name deterministic failure shapes and remediation paths, but they must not describe those examples as solver models unless backed by formal-proof receipts.

**Rationale:** This imports Salt's clarity without importing Salt's verifier claims.

### 3. Use explicit FAQ boundaries instead of marketing caveats

**Choice:** The FAQ should answer comparison questions directly and name supported surfaces and unsupported claims in the same answer.

**Rationale:** A compact FAQ helps readers understand Octet quickly and gives validation a stable place to check boundary language.

## Risks / Trade-offs

- **Duplication drift:** README, consumer guide, and adoption story may repeat quickstart commands. Mitigate with one canonical story page and short links.
- **Overclaim temptation:** A polished card surface may invite stronger proof language. Mitigate with explicit forbidden wording tests.
- **Example brittleness:** Lint behavior may evolve. Mitigate by selecting examples from stable lint families and tying examples to lint names rather than exact diagnostic text.
- **Visual debt:** Markdown cards are less polished than a website. Accept for this change; a future site can consume the same content.

## Validation

- Positive inventory confirms required story sections/cards are present.
- Positive example check confirms each before/after example names lint, failure shape, remediation, and non-claim where needed.
- Positive quickstart check confirms command sequence includes artifact output and artifact verification.
- Positive FAQ check confirms required topics are covered.
- Negative wording check rejects Salt-style claims about language syntax, mathematical certainty, whole-program proof, verified runtime, C parity, and certification.
- Cairn `validate` and proposal/design/tasks gates pass.
