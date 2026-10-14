# Evaluate the May 17 Verus release for stack compatibility

## Why

The April production verifier can prove repaired Trellis source but cannot execute three required-native regression corpora because of removed APIs and a verifier internal error. Verus `0.2026.05.17.e479cce` uses Rust `1.95.0`, passes complete repaired Trellis and unmodified node replication, and supports a bounded, separately identified compatibility route for the two remaining corpora. Selecting it still requires exact native proof evidence rather than package availability.

## What changes

- Package the exact May 17 release, full Verus revision, and Rust closure through Octet's existing production wrapper.
- Run Octet proofs and the closed Trellis compatibility portfolio before selecting it as the production profile.
- Update consumer rows only where exact downstream evidence passes; otherwise retain their prior status.
- Keep source identity, adaptations, verifier execution, assumptions, and downstream receipts as separate evidence classes.

## Success

The profile may select May 17 only if Octet positive and negative proof rails pass, exact package identities are recorded, complete Trellis and every required-native corpus row pass, generated profile freshness passes, and no proof authority is transferred across releases or adapted source boundaries.
