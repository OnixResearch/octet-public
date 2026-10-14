## Why

Octet's proof claim catalog stores claim and alias rows as vectors, then repeatedly builds ad-hoc sets and maps for uniqueness checks and reference resolution. Claim rows already carry `claim_id`, and alias rows carry `alias_id`, so each temporary map duplicates identity that belongs to the row itself.

`iddqd` is a good fit for this catalog because it provides ordered maps whose keys are borrowed from values and forces unique insertion to be explicit. That matches Octet's evidence style: duplicate proof claims and aliases should fail closed, not be overwritten or resolved by scan order.

## What Changes

- Add iddqd-backed ordered indexes for proof claim entries and proof claim aliases.
- Keep the catalog JSON shape as vectors for reviewable diffs and compatibility.
- Resolve claim references through validated indexes rather than repeated vector scans.
- Add positive and negative fixtures for claim ids, aliases, alias-to-claim resolution, duplicate ids, missing targets, self-targets, and invalid tier promotion.

## Impact

- Proof claim validation becomes more explicit about identity and alias uniqueness.
- Public proof-card and proof-ref workflows keep the same artifact shapes.
- This avoids `BiHashMap` because multiple aliases may legitimately target one canonical claim.
