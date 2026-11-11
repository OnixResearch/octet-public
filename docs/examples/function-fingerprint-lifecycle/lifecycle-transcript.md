# Function fingerprint lifecycle smoke

This artifact shows the complete operator lifecycle for one reviewed function:
initial marker repair, clean check, source drift, diff diagnosis, accepted update,
and final clean check.

The fixture was created in a scratch directory, then the final repaired source was
copied into `reviewed.rs`. Commands were run from the scratch directory with the
repo-local binary:

```bash
TOOL='cargo run -q --manifest-path ~/git/octet/cargo-octet/Cargo.toml --'
```

## Initial fixture

```rust
// octet-fingerprint: b3:0; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
#[rustfmt::skip]
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    50_u32.saturating_mul(attempt.saturating_add(1))
}
```

`#[rustfmt::skip]` is an outer attribute, so the marker sits on the line above it
and the attribute is hashed as part of the function item. A marker placed between
an attribute and `fn` does not bind to the function.

## Initial update

```bash
$ $TOOL fingerprint --update reviewed.rs
```

```text
octet fingerprint update
reviewed.rs bounded_retry_delay_ms written semantic_id=b3:b0ce417cdb81a888 stored=0 computed=b3:7174aea81e7352b802272ef69d3708ccbc87f6f632136bb6b85e829b359fc37b normalized_body_hash=b3:ac75e8ed76065675b23915d9b7779a6f8fd620def344a345ec80e5451cc0a73f dependency_identity_status=unsupported-macro-context review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

## Clean check

```bash
$ $TOOL fingerprint --check reviewed.rs
```

```text
octet fingerprint check
reviewed.rs bounded_retry_delay_ms match semantic_id=b3:b0ce417cdb81a888 stored=7174aea81e7352b802272ef69d3708ccbc87f6f632136bb6b85e829b359fc37b computed=b3:7174aea81e7352b802272ef69d3708ccbc87f6f632136bb6b85e829b359fc37b normalized_body_hash=b3:ac75e8ed76065675b23915d9b7779a6f8fd620def344a345ec80e5451cc0a73f dependency_identity_status=unsupported-macro-context review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Drift mutation

The reviewed function body was changed from:

```rust
50_u32.saturating_mul(attempt.saturating_add(1))
```

to:

```rust
75_u32.saturating_mul(attempt.saturating_add(1))
```

## Diff after drift

```bash
$ $TOOL fingerprint --diff reviewed.rs
$ echo $?
1
```

```text
octet fingerprint diff
reviewed.rs bounded_retry_delay_ms mismatch semantic_id=b3:bc2dd43f7aa746d7 stored=7174aea81e7352b802272ef69d3708ccbc87f6f632136bb6b85e829b359fc37b computed=b3:90089d987ce66794a19701cdb27dfecee5a5283ed5d3bd3dd2de2b102fd7edb9 normalized_body_hash=b3:c3d1f40a016b05dea67ebb76d88107a9699d4b989f6aadb0e773b1a3bd515a23 dependency_identity_status=unsupported-macro-context review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Accepted update after review

```bash
$ $TOOL fingerprint --update reviewed.rs
```

```text
octet fingerprint update
reviewed.rs bounded_retry_delay_ms written semantic_id=b3:bc2dd43f7aa746d7 stored=7174aea81e7352b802272ef69d3708ccbc87f6f632136bb6b85e829b359fc37b computed=b3:90089d987ce66794a19701cdb27dfecee5a5283ed5d3bd3dd2de2b102fd7edb9 normalized_body_hash=b3:c3d1f40a016b05dea67ebb76d88107a9699d4b989f6aadb0e773b1a3bd515a23 dependency_identity_status=unsupported-macro-context review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

## Final clean check

```bash
$ $TOOL fingerprint --check reviewed.rs
```

```text
octet fingerprint check
reviewed.rs bounded_retry_delay_ms match semantic_id=b3:bc2dd43f7aa746d7 stored=90089d987ce66794a19701cdb27dfecee5a5283ed5d3bd3dd2de2b102fd7edb9 computed=b3:90089d987ce66794a19701cdb27dfecee5a5283ed5d3bd3dd2de2b102fd7edb9 normalized_body_hash=b3:c3d1f40a016b05dea67ebb76d88107a9699d4b989f6aadb0e773b1a3bd515a23 dependency_identity_status=unsupported-macro-context review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Final fixture

The checked-in `reviewed.rs` file is the final repaired state. The important
operator assertions are:

- clean checks report `match=1 mismatch=0 written=0`;
- drift diff exits `1` and reports `mismatch=1`;
- accepted update reports exactly one `written` marker;
- final check returns to `match=1 mismatch=0 written=0`.
