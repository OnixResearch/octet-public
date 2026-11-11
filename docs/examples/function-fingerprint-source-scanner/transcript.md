# Function fingerprint source-scanner transcript

This transcript documents the source-scanner boundary: `macro_rules!` generated functions are not fingerprint records, while explicit source wrappers are. Commands are run from this directory. The first update starts from the same source with the marker temporarily set to `b3:0`; the checked-in `reviewed.rs` is the final repaired state.

## Update fingerprints only the explicit source wrapper

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs explicit_generated_boundary written semantic_id=b3:c8990f6dd44c3dd4 stored=0 computed=b3:05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d normalized_body_hash=b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
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
      "stored_hash": "05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d",
      "computed_hash": "05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d",
      "normalized_body_hash": "b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88",
      "semantic_id": "b3:c8990f6dd44c3dd4",
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
# exit status: 0
```

## Human check reports one explicit source function

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs explicit_generated_boundary match semantic_id=b3:c8990f6dd44c3dd4 stored=05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d computed=b3:05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d normalized_body_hash=b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
