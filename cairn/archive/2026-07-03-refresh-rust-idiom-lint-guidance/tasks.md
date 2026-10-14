## Contract

- [x] [serial] Specify guidance-freshness behavior for narrative docs that name shipped lints or enforcement tiers. r[octet.lint_guidance_freshness.enforcement_tiers]
- [x] [serial] Specify that shipped lints MUST NOT be described as future-only candidates. r[octet.lint_guidance_freshness.shipped_lints]

## Implementation

- [x] [serial] Update `docs/rust-idiom-guidance.md` so `Borrowed argument type` references the shipped `borrowed_argument_types` warning lint. r[octet.lint_guidance_freshness.shipped_lints]
- [x] [serial] Add a small deterministic docs freshness helper or test fixture for guidance rows naming live lint identities. r[octet.lint_guidance_freshness.enforcement_tiers]

## Verification

- [x] [serial] Add a positive test for the current guidance row and a negative stale-wording fixture. r[octet.lint_guidance_freshness.shipped_lints]
- [x] [serial] Run focused docs freshness tests, `cargo test --lib`, and Cairn validation/gates. r[octet.lint_guidance_freshness.enforcement_tiers]
