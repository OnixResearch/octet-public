## Why

Proof receipts currently bind a configured build-closure manifest, but the manifest is still author-maintained fixture text. A reviewer can see that some build context was accounted for, but not that the receipt names a first-class Nix derivation and recursive closure identity for the verifier artifact.

## What Changes

- Add typed verifier Nix derivation and recursive closure metadata to proof receipt contracts.
- Export those obligations through generated policy artifacts.
- Require receipt-only and command-generated proof rails to include deterministic Nix derivation/closure receipt lines when configured.
- Add a Nix-generated closure manifest package so fixture metadata can be regenerated from the actual `tigerstyle-standards` derivation instead of freehand closure prose.

## Out of Scope

- Remote cache trust policies or signature verification.
- Running Nix from `tigerstyle-standards check`; the checker remains local/read-only unless proof command execution is explicitly requested.
- Downstream Aspen pin updates.

## Verification

Use focused CLI tests, full `tigerstyle-standards` tests, Nickel/Nix policy validation, standards fixtures, docs examples, strict OpenSpec validation, and whitespace checks.
