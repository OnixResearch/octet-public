## Contract

- [x] [serial] Define the first checked Rust patch walkthrough requirement and finding scenario. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Define artifact-producing and artifact-verification command requirements for the walkthrough. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Define onboarding and proof-boundary wording requirements for the walkthrough. r[octet.adoption_story.first_checked_patch_walkthrough]

## Implementation

- [x] [serial] Add the walkthrough at `docs/examples/first-checked-rust-patch/README.md` or an equivalent canonical docs path. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Select one existing high-confidence lint or policy surface and show before code, finding shape, fixed code, and remediation rationale. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Include the artifact-producing `cargo octet check --artifact-dir artifacts/octet` path, expected local outputs, and `cargo octet artifact verify --artifact-dir artifacts/octet`. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Link the walkthrough from `README.md` and `docs/adoption-story.md` without duplicating the long-form content. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Add or extend deterministic docs checks for walkthrough sections, commands, artifacts, lint naming, remediation shape, and forbidden unsupported claims. r[octet.adoption_story.first_checked_patch_walkthrough]

## Verification

- [x] [serial] Run a positive documentation inventory proving the walkthrough names the selected lint or policy surface, before code, deterministic finding shape, fixed code, and remediation rationale. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Run a positive command and artifact inventory proving the walkthrough names artifact-producing check commands, expected local outputs, provenance, and artifact verification. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Run a negative wording check rejecting solver-counterexample, mathematical-certainty, whole-program-proof, certification, C-performance-parity, verified-runtime, Rust-replacement, and Salt-syntax overclaims. r[octet.adoption_story.first_checked_patch_walkthrough]
- [x] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.adoption_story.first_checked_patch_walkthrough]
