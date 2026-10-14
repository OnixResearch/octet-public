# Function Address Export Specification

## Purpose

Expose source byte spans in function-address reports so downstream receipts do not reconstruct Octet scanner state.

## Requirements

### Requirement: Function-address records carry byte spans
r[octet.function_address_export.source_spans.contract] Octet MUST include zero-based `start_byte` and `end_byte` offsets for each exported function-address record.

#### Scenario: Span points into source envelope
r[octet.function_address_export.source_spans.positive]
- GIVEN a function-address record references a source envelope
- WHEN downstream tooling reads the record
- THEN `start_byte..end_byte` MUST slice the referenced source envelope bytes for the scanner-selected function item.

### Requirement: Span validation fails closed
r[octet.function_address_export.source_spans.validation] Octet MUST validate exported spans against source envelope bytes before treating the report as a reviewed baseline or downstream receipt input.

#### Scenario: Bad span is rejected
r[octet.function_address_export.source_spans.negative]
- GIVEN a function-address record has an out-of-range, inverted, stale-source, or non-matching span
- WHEN span validation runs
- THEN Octet MUST emit a deterministic diagnostic naming the invalid record and field.

### Requirement: Byte spans do not imply Rust semantics
r[octet.function_address_export.source_spans.export] Octet MUST state that function-address spans describe source scanner identity only and do not claim HIR, MIR, compiler, or behavioral equivalence.

#### Scenario: Downstream smoke consumes spans directly
r[octet.function_address_export.source_spans.final_validation]
- GIVEN Octet emits function-address records with byte spans
- WHEN Kamacite and Valence build function-address receipts
- THEN they SHOULD consume the exported spans directly without recomputing source offsets.
