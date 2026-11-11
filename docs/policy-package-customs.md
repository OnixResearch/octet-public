# Policy package customs gate

The local customs gate checks policy/config/template/proof-rail/generated-contract packages before promotion.

A package manifest uses `standards/policy-packages/contracts.ncl` and declares package ID, kind, owner, source paths, generated artifacts, registry family IDs, generation and validation commands, fixtures, provenance, license notes, non-claims, and manual review actions. The gate rejects a `kind` outside the contract's `PackageKind` set: `policy`, `profile`, `template`, `proof-rail`, `generated-contract`, or `contract`.

`octet-standards customs-gate` validates the manifest against:

- registry coverage in `standards/policies/registry.ncl`; a `promotable` or `existing` package must name at least one registry family
- positive and negative fixture coverage
- generated artifact freshness through a BLAKE3 manifest; a `promotable` or `existing` `policy`, `profile`, `template`, `generated-contract`, or `contract` package must declare a generated artifact or a deterministic export generation command (`nickel export` or `nickel-export-manifest`)
- checked-in source and fixture inventory
- no remote fetches during validation
- no raw secret material in package sources, positive and negative fixtures, or generated artifacts; a `secret_exclusion_rationale` excuses secret-shaped text only for `non-candidate` packages, so a package whose fixtures need secret-shaped text cannot be promoted

The secret scan is case-insensitive and line-based. It flags PEM private-key headers such as `-----BEGIN RSA PRIVATE KEY-----` or `-----BEGIN OPENSSH PRIVATE KEY-----`, and a credential key name (`password`, `passwd`, `token`, `secret`, `api_key`, `apikey`, or `private_key`) followed by optional quotes and whitespace and then `=` or `:`, so `password="x"`, `api_key = x`, and `{"api_key": "x"}` all fail. Longer names that end in a key name, such as `client_secret = x`, also fail; words such as `tokenizer` or `secretary` do not. The scan is a review aid and does not prove that a package holds no secrets.

`standards/policy-packages/contracts.ncl` `PackageRules` mirrors the fixture, registry-family, and freshness-or-export coverage rules for promotable packages.

The report is deterministic and local. Passing customs does not trust package authors, prove package safety, prove vulnerability freedom, or replace repository review.
