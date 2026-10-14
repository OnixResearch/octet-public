# Function fingerprint source-scanner transcript

This transcript documents the source-scanner boundary: `macro_rules!` generated functions are not fingerprint records, while explicit source wrappers are. Commands are run from this directory. The first update starts from the same source with the marker temporarily set to `b3:0`; the checked-in `reviewed.rs` is the final repaired state.

## Update fingerprints only the explicit source wrapper

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs explicit_generated_boundary written stored=0 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## JSON check reports one explicit source function

```console
$ cargo octet fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "explicit_generated_boundary",
      "stored_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "computed_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "status": "match",
      "review": "source-scanner-boundary",
      "evidence": "docs/examples/function-fingerprint-source-scanner/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Human check reports one explicit source function

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs explicit_generated_boundary match stored=eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
## Regenerated output compatibility

### Output 1

```text
octet fingerprint update
reviewed.rs explicit_generated_boundary written semantic_id=b3:5ea45525e763250d stored=0 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 normalized_body_hash=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

### Output 2

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "explicit_generated_boundary",
      "stored_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "computed_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "normalized_body_hash": "b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "semantic_id": "b3:5ea45525e763250d",
      "dependency_set_hash": "b3:1d4fc535ad0b0fd60b72a85e2e4cc6ed26c80949c16e61c48836697a81111c31",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "match",
      "review": "source-scanner-boundary",
      "evidence": "docs/examples/function-fingerprint-source-scanner/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
```

### Output 3

```text
octet fingerprint check
reviewed.rs explicit_generated_boundary match semantic_id=b3:5ea45525e763250d stored=eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 normalized_body_hash=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

### Output 4

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "explicit_generated_boundary",
      "stored_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "computed_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "normalized_body_hash": "b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "semantic_id": "b3:5ea45525e763250d",
      "dependency_set_hash": "b3:1d4fc535ad0b0fd60b72a85e2e4cc6ed26c80949c16e61c48836697a81111c31",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "match",
      "review": "source-scanner-boundary",
      "evidence": "docs/examples/function-fingerprint-source-scanner/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
```
