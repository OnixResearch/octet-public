# Source-acquisition receipts

Source-acquisition receipts record how stack-owned dependencies and inputs were obtained. They are separate from vulnerability scans, license checks, cargo-vet trust evidence, and formal proof receipts.

Each row names a label, source kind, locator, revision or lock identity, credential boundary, release eligibility, local override policy, and non-claims. The required non-claim is: source acquisition records source identity and policy posture only; it is not dependency audit, vulnerability proof, or formal proof.

## Source kinds

- `registry`: package source from a registry lock identity.
- `public_git`: public Git URL pinned to a revision.
- `private_ssh_git`: private SSH Git URL pinned to a revision and an explicit credential boundary.
- `vendored_snapshot`: local vendored bytes with upstream revision and lock identity.
- `nix_flake`: Nix flake input with locked revision and NAR/source hash.
- `local_override`: local sibling or path override; development-only by default.

## Consumer guidance

Private SSH stack dependencies must record which credential boundary is required and whether that source is release-eligible. Local sibling overrides such as `../valence` or `../trellis` must be marked non-release unless a separate public, vendored, or registry release source identity is declared. Parent links from dependency-trust summaries may cite source-acquisition receipts, but the source evidence tier stays `policy_source_validation` or `manual_review`; it is not merged into dependency-trust or formal-proof evidence.
