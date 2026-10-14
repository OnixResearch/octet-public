# Structured migration session example

Copy `templates/structured-migration-session.json`, edit the pointers and surfaces, then inspect it before any apply step:

```sh
cargo octet migrate session plan --session migration-session.json --output-format json
cargo octet migrate session verify --session migration-session.json --output-format json
cargo octet migrate session apply --session migration-session.json --operation-manifest operation-manifest.json --dry-run --output-format json
```

The session records compatibility aliases and archive boundaries. It does not claim semantic equivalence or correctness; use separate tests or proof receipts for those claims.
