# Function fingerprint warning summary transcript

This transcript is executable documentation for `macro_warnings` and `metadata_missing` summary counts. Commands are run from this directory.

## Initial update writes markers and reports warning counts

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs unreviewed_macro_path written semantic_id=b3:990bb532b9cd5247 stored=0 computed=b3:4bb0a5a53fdcdada0290b6d4f90cca72ad0635e0b535fe2ca09374d8986ab32f normalized_body_hash=b3:47e0d79c9a1f992430033f7ebff03c5a4c876fd107870d120f74959c541b1420 dependency_identity_status=unsupported-macro-context review=macro-warning-example evidence=docs/examples/function-fingerprint-warning-summary/transcript.md macro_warning=true
reviewed.rs missing_gate_metadata written semantic_id=b3:ae0984195641aed0 stored=0 computed=b3:83b2e51ca9854736155771a41c9e116ca71043f78c367f84fe1e4773d299f816 normalized_body_hash=b3:084ea9caf6cf4208c234ed5369740e661dffe1102e55916d267fa3dcaa01fb6b dependency_identity_status=source-scanner-token-set review=<missing> evidence=<missing> macro_warning=false
summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1
# exit status: 0
```

## Clean check fails on warning counts

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs unreviewed_macro_path match semantic_id=b3:990bb532b9cd5247 stored=4bb0a5a53fdcdada0290b6d4f90cca72ad0635e0b535fe2ca09374d8986ab32f computed=b3:4bb0a5a53fdcdada0290b6d4f90cca72ad0635e0b535fe2ca09374d8986ab32f normalized_body_hash=b3:47e0d79c9a1f992430033f7ebff03c5a4c876fd107870d120f74959c541b1420 dependency_identity_status=unsupported-macro-context review=macro-warning-example evidence=docs/examples/function-fingerprint-warning-summary/transcript.md macro_warning=true
reviewed.rs missing_gate_metadata match semantic_id=b3:ae0984195641aed0 stored=83b2e51ca9854736155771a41c9e116ca71043f78c367f84fe1e4773d299f816 computed=b3:83b2e51ca9854736155771a41c9e116ca71043f78c367f84fe1e4773d299f816 normalized_body_hash=b3:084ea9caf6cf4208c234ed5369740e661dffe1102e55916d267fa3dcaa01fb6b dependency_identity_status=source-scanner-token-set review=<missing> evidence=<missing> macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1
# exit status: 1
```
