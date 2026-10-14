# Source boundary core

Octet source-boundary validation is pure, deterministic logic over in-memory descriptors. Shells may gather facts from Git, Cargo, local worktrees, vendored directories, generated artifacts, or registries, but the core owns only descriptor normalization and validation.

Supported descriptor kinds:

- `git`: requires revision identity, selected paths, digest inventory, dirty-state disposition, and non-claims.
- `local override`: allowed for development when marked non-release-eligible and bounded by an explicit dirty-state policy.
- `vendored snapshot`: requires upstream source identity, upstream revision, selected paths, digest inventory, and local-patch disposition.
- `registry package`: requires package identity and digest inventory.
- `generated artifact`: requires generator identity and digest inventory.
- `unknown/manual-review`: fails closed unless a higher-level rail explicitly handles manual review.

Passing source-boundary validation proves descriptor shape, source identity completeness, digest inventory consistency, and non-claim conformance only. It does not prove dependency trust, code correctness, supply-chain safety, release eligibility, verifier soundness, or downstream behavioral correctness.
