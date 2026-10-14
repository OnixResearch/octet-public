## Why

external evidence tooling's strongest transferable move is not its numerical runtime claim; it is the category framing that agent-authored code exceeds human review capacity and therefore needs a narrower checked path before production. Octet's README still leads with "Rust-compatible toolchain and lint distribution," which is accurate but undersells the adoption reason for teams using code-generation agents.

Octet should make the checked path explicit: generated or human-written Rust enters a constrained route where static lints, policy checks, reviewable suppressions, and evidence artifacts shape what can reach CI and release gates. This positions Octet around the workflow it enables without claiming to be a verified runtime or a formal proof system.

## What Changes

- Establish a canonical product sentence: Octet is the evidence-backed checked path for agent-written and human-written Rust systems code.
- Add a short source-to-gate flow to the README and consumer-facing docs: Rust source, Octet checks, policy/config, reviewable evidence, CI/release gate.
- Explain how the Dylint lint distribution, standards checker, runner, and artifacts participate in that path.
- Add explicit guardrails forbidding unqualified "verified runtime" or "proves program correctness" language in Octet positioning.

## Impact

- **Files**: `README.md`, `docs/consumer-guide.md`, `docs/downstream-rollout.md`, and any future public-site copy source if present.
- **Testing**: documentation checks that assert the canonical phrase and flow appear; negative text checks that reject unqualified verified-runtime or correctness-proof claims.
