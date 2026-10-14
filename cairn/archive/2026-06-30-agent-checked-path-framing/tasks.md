## Contract

- [x] [serial] Define the canonical checked-path sentence and target documentation surfaces. r[octet.product_positioning.agent_checked_path]
- [x] [serial] Define forbidden overclaim language for this positioning. r[octet.product_positioning.no_verified_runtime_claim]

## Implementation

- [x] [serial] Update `README.md` introduction to use the canonical checked-path framing. r[octet.product_positioning.agent_checked_path]
- [x] [serial] Add a source-to-gate flow to `README.md` or `docs/consumer-guide.md` naming source Rust, Octet checks, policy/config, evidence artifacts, and CI/release gates. r[octet.product_positioning.agent_checked_path]
- [x] [serial] Update rollout-facing docs to describe agent-written and human-written Rust as sharing the same checked path. r[octet.product_positioning.agent_checked_path]

## Verification

- [x] [serial] Add or run a positive documentation check proving the canonical sentence and flow appear in consumer-facing docs. r[octet.product_positioning.agent_checked_path]
- [x] [serial] Add or run a negative documentation check proving unqualified `verified runtime` and `proves program correctness` claims are absent. r[octet.product_positioning.no_verified_runtime_claim]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.product_positioning.agent_checked_path]
