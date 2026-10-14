# Tasks: Publish a stack Verus toolchain profile

- [x] [serial] Add the typed Nickel Verus toolchain profile, deterministic JSON export, and positive/negative contract fixtures r[octet.verus_proof_rails.toolchain_profile]
- [x] [serial] Register the profile as a Nickel-authored config family and bind the production Verus Nix constants to the exported profile r[octet.verus_proof_rails.toolchain_profile]
- [x] [serial] Package the pinned `verusfmt` release with self-update disabled and identity output r[octet.verus_proof_rails.verusfmt]
- [x] [parallel] Add formatter positive and negative fixtures plus fail-closed Nix checks r[octet.verus_proof_rails.verusfmt.fail_closed]
- [x] [parallel] Format Octet-owned Verus proof sources and require `verusfmt --check --verus-only` r[octet.verus_proof_rails.verusfmt]
- [x] [serial] Document compatibility-row semantics and add upstream references r[octet.verus_proof_rails.toolchain_profile.compatibility]
- [x] [serial] Run profile freshness, policy, formatter, production Verus positive/negative, and Cairn validation checks r[octet.verus_proof_rails.toolchain_profile.final_validation]
