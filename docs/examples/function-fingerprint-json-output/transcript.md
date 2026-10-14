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
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
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
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
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
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
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
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
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
      "stored_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
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
## Regenerated output compatibility

### Output 1

```text
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "0",
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "normalized_body_hash": "b3:d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "semantic_id": "b3:0b0e5d9093b4693c",
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
```

### Output 2

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "normalized_body_hash": "b3:d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "semantic_id": "b3:0b0e5d9093b4693c",
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
```

### Output 3

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "normalized_body_hash": "b3:9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "semantic_id": "b3:2c3127b904340cf3",
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
```

### Output 4

```text
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "normalized_body_hash": "b3:9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "semantic_id": "b3:2c3127b904340cf3",
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
```

### Output 5

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "normalized_body_hash": "b3:9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "semantic_id": "b3:2c3127b904340cf3",
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
```

### Output 6

```text
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "normalized_body_hash": "b3:9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "semantic_id": "b3:2c3127b904340cf3",
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
```
