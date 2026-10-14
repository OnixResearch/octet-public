## Why

Unison's abilities make effects visible in type signatures. Octet already has purity-effect lints and a capability vocabulary, but consumers still need a checked, reviewable declaration of which effect capabilities are allowed in core, shell, adapter, test, and remote-execution scopes. A capability manifest checker turns the vocabulary into an evidence-backed gate: core logic can remain deterministic because filesystem, network, time, randomness, process, environment, secrets, unsafe, global-state, and concurrency authority must be declared, scoped, and handled explicitly.

## What Changes

- Add a policy-owned capability manifest mechanism that declares scope classes, allowed capability families, capability traits/tokens, deterministic test handlers, and shell-owned interpreters.
- Add a checker that compares declared scopes against Octet's existing effect-family observations and purity lints.
- Emit a capability report that maps direct platform effects, injected capabilities, deterministic handlers, and shell interpreters to evidence tiers and replayable artifacts.
- Preserve Rust compatibility: this is a policy/checker surface, not new Rust syntax, not an effect system, and not a transitive whole-program purity proof.

## Impact

- **Files / artifacts**: expected implementation areas are Nickel policy contracts under `standards/policies/`, standards checker or `cargo-octet` report code, docs under `docs/`, templates for consumer manifests, and checked fixtures.
- **Ordering / dependencies**: define the manifest schema before checker behavior; then wire report output and docs/examples.
- **Testing**: positive fixtures for declared core/shell/test scopes, negative fixtures for undeclared direct effects, unknown capability names, missing handlers, malformed policy, and overclaim wording.
- **Compatibility**: consumers with no capability manifest should continue receiving existing lint behavior; strict gating is opt-in until the manifest is declared.
