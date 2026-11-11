# Function fingerprint human summary transcript

This transcript is executable documentation for human-output summary counts. Commands are run from this directory.

## Initial update writes both markers

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs capped_batch_size written semantic_id=b3:f455ab691b8aba95 stored=0 computed=b3:de90c1f562998daef56f50f69baa60309e27f3a121b98504847e29612c2b0553 normalized_body_hash=b3:80d8f638af7919227d5d84a2240d8c25e5921aeb55a68bc554f8368846ed893e dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch written semantic_id=b3:0bc5301274409301 stored=0 computed=b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 normalized_body_hash=b3:893e1751e1d78f2bcce29e68d58fc4713ca86b7925942f58345659d462c10fd8 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Clean check reports two matches

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size match semantic_id=b3:f455ab691b8aba95 stored=de90c1f562998daef56f50f69baa60309e27f3a121b98504847e29612c2b0553 computed=b3:de90c1f562998daef56f50f69baa60309e27f3a121b98504847e29612c2b0553 normalized_body_hash=b3:80d8f638af7919227d5d84a2240d8c25e5921aeb55a68bc554f8368846ed893e dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:0bc5301274409301 stored=e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 computed=b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 normalized_body_hash=b3:893e1751e1d78f2bcce29e68d58fc4713ca86b7925942f58345659d462c10fd8 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Drift check reports one mismatch

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size mismatch semantic_id=b3:c9364aaaa283994a stored=de90c1f562998daef56f50f69baa60309e27f3a121b98504847e29612c2b0553 computed=b3:c2cb392a462f04b9e56590bc9f2941b9724e16647901322fa15c00928af7d687 normalized_body_hash=b3:bf1a790276003a2b820de69adfc014ca2bcbc642cd025f5e1df2b6aeec8fc9b3 dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:0bc5301274409301 stored=e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 computed=b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 normalized_body_hash=b3:893e1751e1d78f2bcce29e68d58fc4713ca86b7925942f58345659d462c10fd8 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 1
```

## Repair update rewrites one marker

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs capped_batch_size written semantic_id=b3:c9364aaaa283994a stored=de90c1f562998daef56f50f69baa60309e27f3a121b98504847e29612c2b0553 computed=b3:c2cb392a462f04b9e56590bc9f2941b9724e16647901322fa15c00928af7d687 normalized_body_hash=b3:bf1a790276003a2b820de69adfc014ca2bcbc642cd025f5e1df2b6aeec8fc9b3 dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:0bc5301274409301 stored=e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 computed=b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 normalized_body_hash=b3:893e1751e1d78f2bcce29e68d58fc4713ca86b7925942f58345659d462c10fd8 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=1 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## Repaired check reports two matches

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs capped_batch_size match semantic_id=b3:c9364aaaa283994a stored=c2cb392a462f04b9e56590bc9f2941b9724e16647901322fa15c00928af7d687 computed=b3:c2cb392a462f04b9e56590bc9f2941b9724e16647901322fa15c00928af7d687 normalized_body_hash=b3:bf1a790276003a2b820de69adfc014ca2bcbc642cd025f5e1df2b6aeec8fc9b3 dependency_identity_status=source-scanner-token-set review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
reviewed.rs describe_batch match semantic_id=b3:0bc5301274409301 stored=e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 computed=b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876 normalized_body_hash=b3:893e1751e1d78f2bcce29e68d58fc4713ca86b7925942f58345659d462c10fd8 dependency_identity_status=unsupported-macro-context review=operator-human-summary evidence=docs/examples/function-fingerprint-human-summary/transcript.md macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
