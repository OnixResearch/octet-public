## Why

The Lattice/Salt page works because a first-time reader sees one small example, runs one command path, and understands the value proposition before reading the architecture. Octet already has the card-oriented adoption story, before/after examples, and artifact verification language, but it still lacks one self-contained walkthrough that starts from an ordinary Rust patch, produces a concrete Octet finding, fixes it, and verifies the saved evidence bundle.

A focused "first checked Rust patch" walkthrough should adapt Salt's tutorial shape without adopting Salt's language-level proof claims. The reader should see existing Rust code enter `cargo octet check`, get an evidence-backed finding, apply a bounded remediation, and run `cargo octet artifact verify` against local artifacts.

## What Changes

- Add an adoption-story contract for a self-contained first checked Rust patch walkthrough.
- Define the walkthrough's required content: before code, Octet command, deterministic finding shape, fixed code, saved artifact names, artifact verification command, and explicit non-claims.
- Implement a docs example under `docs/examples/first-checked-rust-patch/` or an equivalent canonical path.
- Link the walkthrough from the README and adoption story without duplicating long-form content.
- Add deterministic documentation checks for required sections, command inventory, artifact names, and forbidden proof/certification wording.

## Non-Goals

- Do not add new lints, runner commands, receipt schemas, or proof rails.
- Do not introduce Rust syntax such as `requires` or `ensures`.
- Do not claim mathematical certainty, solver counterexamples, whole-program correctness, certification, C performance parity, or a verified runtime.
- Do not depend on live Lattice/Salt pages during validation; external pages are inspiration only.
- Do not make the walkthrough a full consumer onboarding guide; it should point to `docs/consumer-guide.md` for production setup.

## Impact

- **Files:** likely `docs/examples/first-checked-rust-patch/README.md`, `docs/adoption-story.md`, `README.md`, and a focused documentation-check fixture or script.
- **Behavior:** documentation and validation only; no runtime, lint, policy, or artifact schema behavior changes.
- **Testing:** positive documentation inventory for tutorial sections and commands; negative wording check for unsupported Salt-style claims; Cairn validation and gates.
