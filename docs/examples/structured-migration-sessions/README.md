# Structured migration session example

Copy `templates/structured-migration-session.json`. Edit the pointers and surfaces. Then inspect the file before you apply it:

```sh
cargo octet migrate session plan --session migration-session.json --output-format json
cargo octet migrate session verify --session migration-session.json --output-format json
cargo octet migrate session apply --session migration-session.json --operation-manifest operation-manifest.json --dry-run --output-format json
```

The session records compatibility aliases and archive boundaries. It does not prove semantic equivalence or correctness.

Use separate tests or proof receipts for those claims.
