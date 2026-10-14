# Review identity registry

Octet review identity separates mutable names from immutable evidence. Human
labels such as `done-review`, policy names, evidence aliases, and fingerprint
manifest handles are pointers; BLAKE3 content hashes are the review targets.

## Name and hash roles

Use a mutable label when humans need a memorable handle:

```sh
cargo octet roadmap pointer set --registry review-registry.json --name done-review --kind prompt --hash b3:<target> --receipt update.json
```

Resolve the label when validating a fingerprint review manifest:

```sh
cargo octet roadmap pointer inspect --registry review-registry.json --name done-review
```

The inspect output preserves both the human label and the immutable target hash.
A match means the reviewed identity and source body still match the recorded
evidence. It is not a correctness, safety, or verification claim.

## Rollback

Pointer updates write receipts with the previous hash. Roll back a bad pointer
move without mutating either target hash:

```sh
cargo octet roadmap pointer rollback --registry review-registry.json --receipt update.json
```

Pointer history keeps the previous immutable target visible for review. Missing
labels should fail strict validation with the missing label and expected target
kind instead of silently falling back to a moving branch, path, or prose name.
