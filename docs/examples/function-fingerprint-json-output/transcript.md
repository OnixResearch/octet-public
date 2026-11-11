# Function fingerprint JSON output transcript

This transcript is executable documentation for script-facing JSON output. Commands are run from this directory.

## Initial JSON update writes the marker

```console
$ cargo octet fingerprint --update --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "0",
      "computed_hash": "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
      "normalized_body_hash": "b3:08a39243e7958345cd57fbc60fa89296f48250abb747ee64ae7ba78cdb6bae5b",
      "semantic_id": "b3:f30e26140e01e3e7",
      "dependency_set_hash": "b3:61992fe00fd340488786403ac289d8dbc3c6237a469b9c0352a02dee91e7073a",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "written",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Clean JSON check reports one match

```console
$ cargo octet fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
      "computed_hash": "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
      "normalized_body_hash": "b3:08a39243e7958345cd57fbc60fa89296f48250abb747ee64ae7ba78cdb6bae5b",
      "semantic_id": "b3:f30e26140e01e3e7",
      "dependency_set_hash": "b3:61992fe00fd340488786403ac289d8dbc3c6237a469b9c0352a02dee91e7073a",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "match",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Drift JSON check reports one mismatch

```console
$ cargo octet fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
      "computed_hash": "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
      "normalized_body_hash": "b3:9d7d00e696aa416d3628b520808a46eb1f80ab853402d54088e959db416a08a8",
      "semantic_id": "b3:28a4d594d2760515",
      "dependency_set_hash": "b3:61992fe00fd340488786403ac289d8dbc3c6237a469b9c0352a02dee91e7073a",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "mismatch",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 1
```

## Accepted JSON update rewrites the marker

```console
$ cargo octet fingerprint --update --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
      "computed_hash": "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
      "normalized_body_hash": "b3:9d7d00e696aa416d3628b520808a46eb1f80ab853402d54088e959db416a08a8",
      "semantic_id": "b3:28a4d594d2760515",
      "dependency_set_hash": "b3:61992fe00fd340488786403ac289d8dbc3c6237a469b9c0352a02dee91e7073a",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "written",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Accepted JSON check reports one match

```console
$ cargo octet fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
      "computed_hash": "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
      "normalized_body_hash": "b3:9d7d00e696aa416d3628b520808a46eb1f80ab853402d54088e959db416a08a8",
      "semantic_id": "b3:28a4d594d2760515",
      "dependency_set_hash": "b3:61992fe00fd340488786403ac289d8dbc3c6237a469b9c0352a02dee91e7073a",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.0",
      "status": "match",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```
