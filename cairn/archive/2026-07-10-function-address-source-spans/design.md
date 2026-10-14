## Design

The function-address collector already has `FunctionSpan` start/end byte offsets from the scanner. Export those offsets directly in `function-addresses.json` and bind them to the source envelope path and source identity.

### Decisions

1. **Byte spans are authoritative.** `start_byte` and `end_byte` are zero-based offsets into the exact source envelope bytes.
2. **Display positions remain derived.** Existing line/column fields stay for humans; downstream identity uses byte spans plus source identity.
3. **Validation stays pure.** Span integrity is checked over in-memory source text and records; file reads remain in the collection shell.
4. **No parser overclaim.** Spans describe Octet's source scanner view only, not complete Rust HIR/MIR identity.

### Validation shape

Positive tests assert every exported span slices the source bytes used for function address hashing. Negative tests cover out-of-range spans, stale source identity, start greater than end, and spans that no longer hash to the recorded address.
