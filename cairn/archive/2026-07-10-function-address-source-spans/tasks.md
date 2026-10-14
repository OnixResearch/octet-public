## Tasks

- [x] [serial] Add `start_byte` and `end_byte` to function-address records. r[octet.function_address_export.source_spans.contract]
- [x] [serial] Thread scanner byte offsets into the report renderer without recomputing spans downstream. r[octet.function_address_export.source_spans.export]
- [x] [serial] Add pure span integrity checks over source envelope bytes and records. r[octet.function_address_export.source_spans.validation]
- [x] [serial] Add positive fixtures for direct Kamacite/Valence span consumption. r[octet.function_address_export.source_spans.positive]
- [x] [serial] Add negative fixtures for stale, out-of-range, inverted, and non-matching spans. r[octet.function_address_export.source_spans.negative]
- [x] [serial] Rerun focused Octet tests and the cross-repo smoke without adapter-side span recomputation. r[octet.function_address_export.source_spans.final_validation]
