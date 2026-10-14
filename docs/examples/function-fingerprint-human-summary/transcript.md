# Function fingerprint human summary transcript

This transcript is executable documentation for human-output summary counts. Commands are run from this directory.

## Initial update writes both markers

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs capped_batch_size written stored=0 computed=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch written stored=0 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Clean check reports two matches

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size match stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Drift check reports one mismatch

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size mismatch stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 1
```

## Repair update rewrites one marker

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs capped_batch_size written stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Repaired check reports two matches

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size match stored=cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
## Regenerated output compatibility

### Output 1

```text
octet fingerprint update
reviewed.rs capped_batch_size written semantic_id=b3:8f96d7284785aa6e stored=0 computed=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 normalized_body_hash=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch written semantic_id=b3:624213edbef74ed2 stored=0 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 normalized_body_hash=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=0 metadata_missing=0
```

### Output 2

```text
octet fingerprint check
reviewed.rs capped_batch_size match semantic_id=b3:8f96d7284785aa6e stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 normalized_body_hash=b3:c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:624213edbef74ed2 stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 normalized_body_hash=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

### Output 3

```text
octet fingerprint check
reviewed.rs capped_batch_size mismatch semantic_id=b3:d0c537610282ece8 stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a normalized_body_hash=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:624213edbef74ed2 stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 normalized_body_hash=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

### Output 4

```text
octet fingerprint update
reviewed.rs capped_batch_size written semantic_id=b3:d0c537610282ece8 stored=c6e43994b80a9ca688dff4b56b9d90c9c6fab6434bde1507ef4a34f48eeedf86 computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a normalized_body_hash=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:624213edbef74ed2 stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 normalized_body_hash=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

### Output 5

```text
octet fingerprint check
reviewed.rs capped_batch_size match semantic_id=b3:d0c537610282ece8 stored=cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a computed=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a normalized_body_hash=b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:624213edbef74ed2 stored=7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 computed=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 normalized_body_hash=b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```
