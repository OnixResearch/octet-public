## Why

Salt's site is effective because it turns complex verification ideas into a skimmable story: a concrete hero promise, "Built with" cards, a short "How it works" section, before/after examples, a terminal-first quickstart, and a FAQ that admits limits. Octet already has the underlying substance: Rust-compatible checked-path positioning, case-study field requirements, methodology gates, evidence tiers, scoped Verus proof receipts, and artifact verification. Those pieces are accurate but still read like reference documentation rather than a first-pass product story.

Octet should adapt Salt's presentation mechanics while preserving Octet's own claim boundary. The public story should show how existing Rust code moves through checks, evidence, and release gates without implying Salt-style language contracts, mathematical certainty, C-performance parity, or a verified runtime.

## What Changes

- Add a card-oriented public story surface for Octet that makes the checked path, evidence stack, optional proof lane, and artifact verification visible at a glance.
- Add "How it works" cards that summarize static lint edge, Nickel policy layer, evidence chain, scoped proof receipts, and controlled deployment.
- Add before/after examples that compare ordinary Rust failure shapes with Octet-checked remediation paths for high-confidence lint families.
- Tighten the first-run quickstart into a copy-paste path with expected outputs and artifact verification.
- Add a compact FAQ covering Clippy comparison, proof boundaries, Dylint/nightly tradeoffs, data locality, starter-profile noise, and when to use `cargo octet prove`.
- Add deterministic documentation checks that inventory the cards/examples/FAQ and reject Salt-style overclaims.

## Non-Goals

- Do not add a public website implementation or visual design system in this change.
- Do not introduce new Rust syntax such as `requires` or `ensures`.
- Do not claim mathematical certainty, whole-program correctness, C performance parity, certification, or a verified runtime.
- Do not add new lints, proof rails, receipt schemas, or runner commands.
- Do not fetch or depend on Salt pages during validation; Salt is inspiration, not a build input.

## Impact

- **Files**: likely `README.md`, `docs/adoption-story.md`, `docs/trust-boundary-evidence.md`, `docs/lint-reference.md`, `docs/consumer-guide.md`, and deterministic doc inventory fixtures or tests.
- **Behavior**: documentation and validation only; no runtime or lint behavior changes.
- **Testing**: positive inventory for required story cards, examples, quickstart outputs, and FAQ topics; negative wording checks for unsupported Salt-style claims; Cairn validation and gates.
