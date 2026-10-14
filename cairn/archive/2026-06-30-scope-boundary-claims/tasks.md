## Contract

- [x] [serial] Define required unsupported-claim boundaries for Octet docs. r[octet.scope_boundary.unsupported_claims]
- [x] [serial] Define required supported-capability statements paired with those boundaries. r[octet.scope_boundary.supported_capabilities]

## Implementation

- [x] [serial] Add a "What Octet does not claim" section to `README.md` or the canonical consumer guide. r[octet.scope_boundary.unsupported_claims]
- [x] [serial] Cross-link the boundary section from safety-critical and release/operator docs. r[octet.scope_boundary.unsupported_claims]
- [x] [serial] Pair each unsupported claim with supported Octet capabilities and artifact surfaces. r[octet.scope_boundary.supported_capabilities]

## Verification

- [x] [serial] Add or run a positive documentation check for required unsupported-claim and supported-capability text. r[octet.scope_boundary.unsupported_claims]
- [x] [serial] Add or run a negative documentation check rejecting unqualified correctness, safety-guarantee, and business-validation claims. r[octet.scope_boundary.unsupported_claims]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.scope_boundary.supported_capabilities]
