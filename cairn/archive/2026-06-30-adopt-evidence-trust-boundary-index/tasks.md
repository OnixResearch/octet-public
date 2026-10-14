## Contract

- [x] [serial] Define the required index themes and link targets. r[octet.trust_boundary_index.themes]
- [x] [serial] Define unsupported-claim restrictions for the index. r[octet.trust_boundary_index.no_overclaims]

## Implementation

- [x] [serial] Add the trust-boundary or evidence-path index to README, consumer guide, or a dedicated docs page. r[octet.trust_boundary_index.themes]
- [x] [serial] Link each theme to a concrete Octet doc section, command, artifact guide, or planned change output. r[octet.trust_boundary_index.themes]

## Verification

- [x] [serial] Add or run a positive link/heading inventory for all six required themes. r[octet.trust_boundary_index.themes]
- [x] [serial] Add or run a negative documentation check rejecting unqualified verified-runtime, proof-stack, and correctness-guarantee claims in the index. r[octet.trust_boundary_index.no_overclaims]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.trust_boundary_index.themes]
